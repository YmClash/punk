1 - LEXER

        1: Faire appel à un analyseur lexical pour transformer le code source en une séquence de tokens.
        2: Tokenise le code source en une séquence de tokens.
        3: utilise une hashmap pour les mots clés, les opérateurs, les delimiters, etc.
        4: Chaque token représente une unité de syntaxe, comme un mot-clé, un identifiant, un opérateur, etc.
        5: Les tokens sont ensuite utilisés par le parser pour construire l'AST.

        Done 
    

2 - PARSER AST
    
        1: Faire appel à un analyseur syntaxique pour transformer la séquence de tokens en une structure de données appelée arbre syntaxique abstrait (AST).
        2: L'AST représente la structure du programme.
        3: L'AST est utilisé par le générateur de code pour produire du code exécutable.
        4: L'AST est une structure de données arborescente qui représente la structure du programme.
        5: Chaque nœud de l'AST représente une unité de syntaxe, comme une déclaration, une expression, une fonction, etc.


        Done 

    

3 - SEMANTIQUE ANALYSE

        1: Faire appel à un analyseur sémantique pour vérifier la validité du programme.
        2: L'analyseur sémantique vérifie les règles sémantiques du langage.
        3: Il vérifie que les variables sont définies avant d'être utilisées, que les types sont compatibles, etc.
        4: L'analyseur sémantique est utilisé pour détecter les erreurs de programmation avant l'exécution du programme.
        5: L'analyseur sémantique est utilisé pour vérifier la validité du programme et générer des messages d'erreur en cas de problème.




4 - EMITTER

        1: Faire appel à un émetteur pour générer du code exécutable à partir de l'AST.
        2: L'émetteur transforme l'AST en code exécutable.
        3: L'émetteur peut générer du code Rust, du bytecode, ou même du code machine.
        4: L'émetteur est utilisé pour produire du code exécutable à partir de l'AST.
        5: L'émetteur est utilisé pour générer du code exécutable à partir de l'AST.




5 - COMPILER






keyword note : 
