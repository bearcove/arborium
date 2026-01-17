// Apache Groovy Example - Basic Syntax Features
// Demonstrates classes, properties, methods, closures, and operators

package com.example

import java.util.concurrent.TimeUnit

/**
 * A Person class demonstrating Groovy's concise syntax
 */
class Person {
    String firstName
    String lastName
    int age

    // Constructor with named parameters
    Person(Map params) {
        this.firstName = params.firstName
        this.lastName = params.lastName
        this.age = params.age ?: 0
    }

    // Method with string interpolation
    String greet() {
        return "Hello, I'm ${firstName} ${lastName} and I'm ${age} years old"
    }

    // Operator overloading
    Person plus(int years) {
        return new Person(
            firstName: this.firstName,
            lastName: this.lastName,
            age: this.age + years
        )
    }
}

// Closures and higher-order functions
def numbers = [1, 2, 3, 4, 5]
def doubled = numbers.collect { it * 2 }
def evens = numbers.findAll { it % 2 == 0 }

println "Original: ${numbers}"
println "Doubled: ${doubled}"
println "Even numbers: ${evens}"

// Safe navigation operator
def person = new Person(firstName: 'Alice', lastName: 'Smith', age: 30)
println person?.greet()

// Elvis operator for null-safety
def name = null
def displayName = name ?: 'Anonymous'
println "Display name: ${displayName}"

// Ranges and iteration
def range = 1..5
range.each { num ->
    println "Number: ${num}"
}

// Maps with various syntaxes
def config = [
    host: 'localhost',
    port: 8080,
    'timeout-ms': 5000
]

// Multiline strings with triple quotes
def sql = """
    SELECT *
    FROM users
    WHERE age > ${person.age}
    ORDER BY name
"""

// GString expressions
def message = "The config uses port ${config.port} on ${config.host}"
println message

// Method references and method pointers
def toUpperCase = String.&toUpperCase
println toUpperCase('groovy')

// Spread operator
def moreNumbers = [6, 7, 8]
def combined = [*numbers, *moreNumbers]
println "Combined: ${combined}"

// Switch with pattern matching
def describe(obj) {
    switch(obj) {
        case String:
            return "String: ${obj}"
        case Integer:
            return "Integer: ${obj}"
        case List:
            return "List with ${obj.size()} elements"
        default:
            return "Unknown type"
    }
}

println describe("hello")
println describe(42)
println describe([1, 2, 3])

// Try-catch with multiple catch blocks
try {
    def result = 10 / 0
} catch (ArithmeticException e) {
    println "Cannot divide by zero"
} catch (Exception e) {
    println "General error: ${e.message}"
} finally {
    println "Cleanup code"
}

// Regex patterns
def pattern = ~/\d{3}-\d{4}/
def text = "Call me at 555-1234"
def matcher = text =~ pattern
if (matcher) {
    println "Found phone number: ${matcher[0]}"
}

// Groovy truth (automatic boolean coercion)
if (numbers) {
    println "List is not empty"
}

if (person?.age) {
    println "Person has a valid age"
}

// Meta-programming example
String.metaClass.shout = { -> delegate.toUpperCase() + '!' }
println "groovy".shout()
