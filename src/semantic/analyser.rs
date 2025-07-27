//src/semantic/analyzer.rs

use crate::parser::ast::{ASTNode, Statement, Declaration, Expression, VariableDeclaration,
                         FunctionDeclaration, Mutability as ASTMutability};
use crate::semantic::borrow_checker::MutabilityManager;
use crate::semantic::symbols::{SymbolKind, SourceLocation};
use crate::semantic::symbol_table::SymbolTable;
use crate::semantic::type_checker::TypeChecker;
use crate::semantic::types::type_system::Mutability;
use crate::semantic::semantic_error::{SemanticError, SemanticErrorType, Position};

/// Analyseur sémantique principal qui coordonne tous les composants
pub struct SemanticAnalyzer {
    pub symbol_table: SymbolTable,
    pub type_checker: TypeChecker,
    pub errors: Vec<SemanticError>,
    pub warnings: Vec<SemanticError>, // Pour les avertissements non-fatals
}

impl SemanticAnalyzer {
    /// Crée un nouvel analyseur sémantique
    pub fn new() -> Self {
        let symbol_table = SymbolTable::new();
        let type_checker = TypeChecker::new(symbol_table.clone());

        SemanticAnalyzer {
            symbol_table,
            type_checker,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Analyse un AST complet
    pub fn analyze(&mut self, ast: &[ASTNode]) -> Result<(), Vec<SemanticError>> {
        // Réinitialiser les erreurs
        println!("Reunitialisation des erreur ");
        self.errors.clear();
        self.warnings.clear();

        // 1. Première passe: déclarer tous les symboles de haut niveau
        println!("Première passe: déclaration des symboles de haut niveau");
        self.declare_top_level_symbols(ast);

        // 2. Deuxième passe: vérifier les types et la sémantique
        println!("Deuxième passe: vérification des types et de la sémantique");
        self.check_semantics(ast);

        // 3. Troisième passe: validations finales
        println!("Troisième passe: validations finales");
        self.final_validations();

        // Retourner les erreurs s'il y en a
        if !self.errors.is_empty() {
            Err(self.errors.clone())
        } else {
            Ok(())
        }
    }

    /// Première passe: déclare tous les symboles de haut niveau
    fn declare_top_level_symbols(&mut self, ast: &[ASTNode]) {
        for node in ast {
            if let Err(error) = self.declare_node_symbols(node) {
                self.errors.push(error);
            }
        }
    }

    /// Déclare les symboles pour un nœud AST
    fn declare_node_symbols(&mut self, node: &ASTNode) -> Result<(), SemanticError> {
        match node {
            ASTNode::Declaration(Declaration::Variable(var_decl)) => {
                self.declare_variable_symbol(var_decl)
            },

            ASTNode::Declaration(Declaration::Function(func_decl)) => {
                self.declare_function_symbol(func_decl)
            },

            ASTNode::Declaration(Declaration::Structure(struct_decl)) => {
                // Créer un nouveau type pour la structure
                let struct_type_id = self.symbol_table.type_system_mut()
                    .type_registry.register_type(
                    crate::semantic::types::type_system::TypeKind::Named(
                        struct_decl.name.clone(),
                        Vec::new()
                    )
                );

                // Déclarer le symbole de la structure
                let location = SourceLocation {
                    file: "current_file.pk".to_string(), // À remplacer par le vrai nom de fichier
                    line: 1, // À remplacer par la vraie position
                    column: 1,
                };

                self.symbol_table.declare_symbol_with_type(
                    struct_decl.name.clone(),
                    SymbolKind::Struct,
                    struct_type_id,
                    location,
                    false // Les types ne sont pas mutables
                )?;

                Ok(())
            },

            ASTNode::Declaration(Declaration::Enum(enum_decl)) => {
                // Créer un nouveau type pour l'énumération
                let enum_type_id = self.symbol_table.type_system_mut()
                    .type_registry.register_type(
                    crate::semantic::types::type_system::TypeKind::Named(
                        enum_decl.name.clone(),
                        Vec::new()
                    )
                );

                let location = SourceLocation {
                    file: "current_file.rs".to_string(),
                    line: 1,
                    column: 1,
                };

                self.symbol_table.declare_symbol_with_type(
                    enum_decl.name.clone(),
                    SymbolKind::Enum,
                    enum_type_id,
                    location,
                    false
                )?;

                Ok(())
            },

            ASTNode::Declaration(Declaration::Trait(trait_decl)) => {
                let location = SourceLocation {
                    file: "current_file.rs".to_string(),
                    line: 1,
                    column: 1,
                };

                self.symbol_table.declare_symbol(
                    trait_decl.name.clone(),
                    SymbolKind::Trait,
                    location
                )?;

                Ok(())
            },

            ASTNode::Declaration(Declaration::Module(module_decl)) => {
                let location = SourceLocation {
                    file: "current_file.rs".to_string(),
                    line: 1,
                    column: 1,
                };

                self.symbol_table.declare_symbol(
                    module_decl.name.clone(),
                    SymbolKind::Module,
                    location
                )?;

                Ok(())
            },

            ASTNode::Statement(_) | ASTNode::Expression(_) => {
                // Les statements et expressions ne déclarent pas de symboles de haut niveau
                Ok(())
            },

            ASTNode::Program(nodes) => {
                // Récursivement déclarer les symboles des sous-nœuds
                for sub_node in nodes {
                    self.declare_node_symbols(sub_node)?;
                }
                Ok(())
            },

            ASTNode::Error(_) => {
                // Ignorer les nœuds d'erreur
                Ok(())
            },

            _ => {
                // Autres types de déclarations non gérés pour l'instant
                Ok(())
            }
        }
    }

    /// Déclare un symbole de variable
    fn declare_variable_symbol(&mut self, var_decl: &VariableDeclaration) -> Result<(), SemanticError> {
        let location = SourceLocation {
            file: "current_file.rs".to_string(),
            line: 1,
            column: 1,
        };

        let is_mutable = matches!(var_decl.mutability, ASTMutability::Mutable);

        // Si un type est spécifié, le convertir
        let type_id = if let Some(ast_type) = &var_decl.variable_type {
            Some(self.symbol_table.type_system_mut().type_registry.convert_ast_type(ast_type))
        } else {
            None
        };

        // Déclarer le symbole
        let symbol_id = if let Some(type_id) = type_id {
            self.symbol_table.declare_symbol_with_type(
                var_decl.name.clone(),
                SymbolKind::Variable,
                type_id,
                location,
                is_mutable
            )?
        } else {
            // Variable sans type explicite - sera inféré plus tard
            let symbol_id = self.symbol_table.declare_symbol(
                var_decl.name.clone(),
                SymbolKind::Variable,
                location
            )?;

            // Marquer comme mutable si nécessaire
            if let Some(symbol) = self.symbol_table.get_symbol_mut(symbol_id) {
                symbol.attributes.is_mutable = is_mutable;
            }

            symbol_id
        };

        Ok(())
    }

    /// Déclare un symbole de fonction
    fn declare_function_symbol(&mut self, func_decl: &FunctionDeclaration) -> Result<(), SemanticError> {
        let location = SourceLocation {
            file: "current_file.rs".to_string(),
            line: 1,
            column: 1,
        };

        // Créer le type de la fonction
        let mut param_type_ids = Vec::new();
        for param in &func_decl.parameters {
            let param_type_id = self.symbol_table.type_system_mut()
                .type_registry.convert_ast_type(&param.parameter_type);
            param_type_ids.push(param_type_id);
        }

        let return_type_id = match &func_decl.return_type {
            Some(ast_type) => self.symbol_table.type_system_mut()
                .type_registry.convert_ast_type(ast_type),
            None => self.symbol_table.type_system().type_registry.type_unit,
        };

        let function_type_id = self.symbol_table.type_system_mut()
            .type_registry.create_function_type(param_type_ids, return_type_id);

        // Déclarer le symbole de la fonction
        self.symbol_table.declare_symbol_with_type(
            func_decl.name.clone(),
            SymbolKind::Function,
            function_type_id,
            location,
            false // Les fonctions ne sont pas mutables
        )?;

        Ok(())
    }

    /// Deuxième passe: vérifier les types et la sémantique
    fn check_semantics(&mut self, ast: &[ASTNode]) {
        for node in ast {
            if let Err(error) = self.check_node_semantics(node) {
                self.errors.push(error);
            }
        }
    }

    /// Vérifie la sémantique d'un nœud AST
    fn check_node_semantics(&mut self, node: &ASTNode) -> Result<(), SemanticError> {
        match node {
            ASTNode::Declaration(declaration) => {
                self.check_declaration_semantics(declaration)
            },

            ASTNode::Statement(statement) => {
                // Synchroniser le type checker avec la table des symboles
                self.sync_type_checker();
                self.type_checker.check_statement(statement)
            },

            ASTNode::Expression(expression) => {
                // Synchroniser le type checker avec la table des symboles
                self.sync_type_checker();
                self.type_checker.check_expression(expression)?;
                Ok(())
            },

            ASTNode::Program(nodes) => {
                // Récursivement vérifier la sémantique des sous-nœuds
                for sub_node in nodes {
                    self.check_node_semantics(sub_node)?;
                }
                Ok(())
            },

            ASTNode::Error(_) => {
                // Ignorer les nœuds d'erreur
                Ok(())
            },
        }
    }

    /// Vérifie la sémantique d'une déclaration
    fn check_declaration_semantics(&mut self, declaration: &Declaration) -> Result<(), SemanticError> {
        match declaration {
            Declaration::Variable(var_decl) => {
                self.check_variable_declaration_semantics(var_decl)
            },

            Declaration::Function(func_decl) => {
                self.check_function_declaration_semantics(func_decl)
            },

            Declaration::Structure(struct_decl) => {
                // Vérifier les champs de la structure
                for field in &struct_decl.fields {
                    // Vérifier que le type du champ existe
                    let field_type_id = self.symbol_table.type_system_mut()
                        .type_registry.convert_ast_type(&field.field_type);

                    if self.symbol_table.type_system().type_registry.get_type(field_type_id).is_none() {
                        return Err(SemanticError::new(
                            SemanticErrorType::TypeError(
                                crate::semantic::semantic_error::TypeError::TypeNotFound(
                                    format!("Type for field {}", field.name)
                                )
                            ),
                            "Invalid field type".to_string(),
                            Position { index: 0 }
                        ));
                    }
                }
                Ok(())
            },

            Declaration::Enum(enum_decl) => {
                // Vérifier les variantes de l'énumération
                for variant in &enum_decl.variantes {
                    // Vérifier que le type de la variante existe
                    let variant_type_id = self.symbol_table.type_system_mut()
                        .type_registry.convert_ast_type(&variant.variante_type);

                    if self.symbol_table.type_system().type_registry.get_type(variant_type_id).is_none() {
                        return Err(SemanticError::new(
                            SemanticErrorType::TypeError(
                                crate::semantic::semantic_error::TypeError::TypeNotFound(
                                    format!("Type for variant {}", variant.name)
                                )
                            ),
                            "Invalid variant type".to_string(),
                            Position { index: 0 }
                        ));
                    }
                }
                Ok(())
            },

            Declaration::Trait(_) => {
                // Pour l'instant, accepter toutes les déclarations de traits
                Ok(())
            },

            _ => {
                // Autres types de déclarations non gérés pour l'instant
                Ok(())
            }
        }
    }

    /// Vérifie la sémantique d'une déclaration de variable
    fn check_variable_declaration_semantics(&mut self, var_decl: &VariableDeclaration) -> Result<(), SemanticError> {
        // Synchroniser le type checker
        self.sync_type_checker();

        // Vérifier la déclaration
        let inferred_type_id = self.type_checker.check_variable_declaration(var_decl)?;

        // Mettre à jour le type du symbole si nécessaire
        if let Ok(symbol_id) = self.symbol_table.lookup_symbol(&var_decl.name) {
            if self.symbol_table.get_symbol_type_id(symbol_id)?.is_none() {
                self.symbol_table.set_symbol_type(symbol_id, inferred_type_id)?;
            }

            // Si la variable a un initializer, la marquer comme initialisée
            if var_decl.value.is_some() {
                self.symbol_table.mark_initialized(symbol_id)?;
            }
        }

        Ok(())
    }

    /// Vérifie la sémantique d'une déclaration de fonction
    fn check_function_declaration_semantics(&mut self, func_decl: &FunctionDeclaration) -> Result<(), SemanticError> {
        // Entrer dans un nouveau scope pour la fonction
        let function_scope_id = self.symbol_table.enter_scope(crate::semantic::symbols::ScopeKind::Function);

        // Déclarer les paramètres dans le scope de la fonction
        for param in &func_decl.parameters {
            let param_type_id = self.symbol_table.type_system_mut()
                .type_registry.convert_ast_type(&param.parameter_type);

            let location = SourceLocation {
                file: "current_file.rs".to_string(),
                line: 1,
                column: 1,
            };

            let param_symbol_id = self.symbol_table.declare_symbol_with_type(
                param.name.clone(),
                SymbolKind::Variable,
                param_type_id,
                location,
                false // Les paramètres sont immutables par défaut
            )?;

            // Marquer le paramètre comme initialisé
            self.symbol_table.mark_initialized(param_symbol_id)?;
        }

        // Synchroniser le type checker
        self.sync_type_checker();

        // Analyser le corps de la fonction
        for stmt_node in &func_decl.body {
            match stmt_node {
                ASTNode::Statement(statement) => {
                    self.type_checker.check_statement(statement)?;
                },
                ASTNode::Expression(expression) => {
                    self.type_checker.check_expression(expression)?;
                },
                ASTNode::Declaration(declaration) => {
                    self.check_declaration_semantics(declaration)?;
                },
                _ => {
                    // Ignorer les autres types de nœuds pour l'instant
                }
            }
        }

        // Sortir du scope de la fonction
        self.symbol_table.exit_scope()?;

        Ok(())
    }

    /// Synchronise le type checker avec la table des symboles actuelle
    fn sync_type_checker(&mut self) {
        // Copier la table des symboles vers le type checker
        // Note: Dans une vraie implémentation, on utiliserait des références partagées
        self.type_checker.symbol_table = self.symbol_table.clone();
        self.type_checker.type_system = self.symbol_table.type_system().clone();
    }

    /// Troisième passe: validations finales
    fn final_validations(&mut self) {
        // Vérifier les références non résolues
        if let Err(validation_errors) = self.symbol_table.validate_references() {
            self.errors.extend(validation_errors);
        }

        // Vérifier les symboles non utilisés (avertissements)
        let unused_symbols = self.symbol_table.check_unused_symbols();
        for symbol_id in unused_symbols {
            if let Ok(symbol) = self.symbol_table.get_symbol(symbol_id) {
                // Créer un avertissement pour les symboles non utilisés
                let warning = SemanticError::new(
                    SemanticErrorType::SymbolError(
                        crate::semantic::semantic_error::SymbolError::SymbolNotFound(
                            format!("Unused symbol: {}", symbol.name)
                        )
                    ),
                    format!("Symbol '{}' is declared but never used", symbol.name),
                    Position { index: 0 }
                );
                self.warnings.push(warning);
            }
        }

        // Vérifier la cohérence du borrow checker
        if let Err(borrow_errors) = self.symbol_table.borrow_checker.validate_all_borrows() {
            // Note: cette méthode n'existe pas encore dans le borrow checker
            // Elle devrait être ajoutée pour vérifier la cohérence finale
        }
    }

    /// Retourne les erreurs collectées
    pub fn get_errors(&self) -> &[SemanticError] {
        &self.errors
    }

    /// Retourne les avertissements collectés
    pub fn get_warnings(&self) -> &[SemanticError] {
        &self.warnings
    }

    /// Vérifie si l'analyse a réussi (pas d'erreurs fatales)
    pub fn is_successful(&self) -> bool {
        self.errors.is_empty()
    }

    /// Retourne des statistiques sur l'analyse
    pub fn get_analysis_stats(&self) -> AnalysisStats {
        AnalysisStats {
            total_symbols: self.symbol_table.symbols.len(),
            total_scopes: self.symbol_table.scopes.len(),
            total_types: self.symbol_table.type_system().type_registry.types.len(),
            error_count: self.errors.len(),
            warning_count: self.warnings.len(),
        }
    }

    /// Analyse une expression isolée (utile pour les tests)
    pub fn analyze_expression(&mut self, expr: &Expression) -> Result<crate::semantic::types::type_system::TypeId, SemanticError> {
        self.sync_type_checker();
        self.type_checker.check_expression(expr)
    }

    /// Analyse un statement isolé (utile pour les tests)
    pub fn analyze_statement(&mut self, stmt: &Statement) -> Result<(), SemanticError> {
        self.sync_type_checker();
        self.type_checker.check_statement(stmt)
    }
}

/// Statistiques sur l'analyse sémantique
#[derive(Debug, Clone)]
pub struct AnalysisStats {
    pub total_symbols: usize,
    pub total_scopes: usize,
    pub total_types: usize,
    pub error_count: usize,
    pub warning_count: usize,
}

// Extension pour valider tous les emprunts dans le borrow checker
impl crate::semantic::borrow_checker::BorrowChecker {
    /// Valide la cohérence de tous les emprunts
    pub fn validate_all_borrows(&self) -> Result<(), Vec<SemanticError>> {
        let mut errors = Vec::new();

        // Vérifier qu'il n'y a pas d'emprunts conflictuels
        for (symbol_id, borrows) in &self.active_borrows {
            let mutable_borrows: Vec<_> = borrows.iter()
                .filter(|b| matches!(b.kind, crate::semantic::borrow_checker::BorrowKind::Mutable |
                                           crate::semantic::borrow_checker::BorrowKind::Write))
                .collect();

            let immutable_borrows: Vec<_> = borrows.iter()
                .filter(|b| matches!(b.kind, crate::semantic::borrow_checker::BorrowKind::Immutable |
                                           crate::semantic::borrow_checker::BorrowKind::Read))
                .collect();

            // Vérifier qu'il n'y a pas de conflit entre emprunts mutables et immutables
            if !mutable_borrows.is_empty() && !immutable_borrows.is_empty() {
                errors.push(SemanticError::new(
                    SemanticErrorType::TypeError(
                        crate::semantic::semantic_error::TypeError::TypeMismatch(
                            format!("Conflicting borrows for symbol {:?}", symbol_id)
                        )
                    ),
                    "Cannot have both mutable and immutable borrows".to_string(),
                    Position { index: 0 }
                ));
            }

            // Vérifier qu'il n'y a pas plus d'un emprunt mutable
            if mutable_borrows.len() > 1 {
                errors.push(SemanticError::new(
                    SemanticErrorType::TypeError(
                        crate::semantic::semantic_error::TypeError::TypeMismatch(
                            format!("Multiple mutable borrows for symbol {:?}", symbol_id)
                        )
                    ),
                    "Cannot have multiple mutable borrows".to_string(),
                    Position { index: 0 }
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}