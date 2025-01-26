# PyRustLang to PunkLang


# PunkLang

PUNK est un langage de programmation expérimental qui combine des éléments de Python et de Rust. Ce projet est un hobby et vise à explorer les concepts de conception de langages de programmation en utilisant Rust comme compilateur backend pour générer du code tout en offrant une syntaxe de haut niveau

## Table des matières

- [Introduction](#introduction)
- [Installation](#installation)
- [Utilisation](#utilisation)
- [Exemples](#exemples)
- [Contribution](#contribution)
- [Licence](#licence)

## Introduction


Ce projet est un hobby et vise à explorer les concepts de conception de langages de programmation en utilisant Rust comme compilateur et Back-end.

PyRust est conçu pour être un langage de programmation simple et expressif, inspiré par la syntaxe de Python et la performance de Rust. 

## Installation
a venir 



## Utilisation
a venir 


## Exemples

Voici un exemple de programme simple en PunkLang :

```punk
les keywords peuvent changer à l'avenir 

# Comprendre les Différences entre les Modes

## Mode Braces :
Utilise des accolades {} pour délimiter les blocs.
Les séparateurs (comme ;) sont utilisés pour terminer les instructions.
## Mode Indentation :
L'indentation (espaces ou tabulations) définit la structure des blocs.
Pas besoin d'accolades.
Les nouvelles lignes et l'indentation remplacent les séparateurs.


#Syntaxe_mode = Indentation

# Ceci est un commentaire

# Déclaration de variables avec typage optionnel
let x: int = 10
let mut y: int = 20

# Fonction simple
fn add(a, b):
    return a + b

# Déclaration de fonction avec typage optionnel
fn add(a: int, b: int) -> int:
    let result: int = a + b
    return result

# Appel de fonction
let sum: int = add(x, y)

# Impression de résultat
print("La somme est: ", sum)

# Structure de contrôle
if x > y:
    print("x est plus grand que y")
elif x < y:
    print("x est plus petit que y")
else:
    print("x est égal à y")

# Boucle for
for i in 5:
    print("i vaut: ", i)

# Boucle while
let count: int = 0
while count < 5:
    print("count vaut: ", count)
    let count = count + 1

# Boucle loop
loop:
    print("Boucle infinie")
    break

# Instruction match
match x:
    10 => print("x vaut 10")
    20 => print("x vaut 20")
    _ => print("x vaut autre chose")

# Importation de module
import math // ou use match
from math import sqrt

# Utilisation d'opérateurs logiques
if is_even(x) && y > 5:
    print("x est pair et y est supérieur à 5")

# Utilisation d'opérateurs bit à bit
let z = x & y

# Déclaration de variable mutuelle avec type inféré
let mut result = calculate_area(7.5, 2.0)

# Utilisation de l'importation
let root: float = sqrt(16)
print("La racine carrée de 16 est: ", root)


/////////////////////////////MONKEY/D/RUST/////OR /////PUNKLANG//////////////////////////


#Syntaxe_mode = Braces

// Ceci est un commentaire


// Déclaration d'une constante et d'une variable mutable
const PI: float = 3.1415;
let mut radius: float = 5.0;

// Fonction pour calculer l'aire d'un cercle
fn calculate_area(r: float) -> float {
    return PI * r * r;
}

// Appel de la fonction
let area: float = calculate_area(radius);


// Déclaration d'une structure
struct Person {
    name: string,
    age: int
}

// Création d'une instance de Person
let person1: Person = Person("Alice", 30);

// Emprunt immuable
let name_ref: &string = &person1.name;
print("Nom de la personne : ", name_ref);

// Emprunt mutable
let mut age_ref: &mut int = &mut person1.age;
*age_ref = 31;
print("Nouvel âge de la personne : ", person1.age);



// Fonction prenant un emprunt mutable
fn increment_value(x: &mut int) {
    *x = *x + 1;
}

// Utilisation de la fonction avec un emprunt mutable
let mut num: int = 10;
increment_value(&mut num);
print("Valeur après incrément : ", num);



// Déclaration d'une fonction avec une clôture
fn apply_twice(f: |int| -> int, x: int) -> int {
    return f(f(x));
}

// Déclaration d'une clôture
let double = |x: int| -> int { x * 2 };



// Utilisation de la fonction avec la clôture
let result: int = apply_twice(double, 5);
println("Résultat : ", result);

