// Groovy Builder Pattern and DSL Example
// Demonstrates Groovy's powerful DSL capabilities used in Gradle and other tools

import groovy.xml.MarkupBuilder

// Email Builder DSL
class EmailBuilder {
    String to
    String from
    String subject
    String body
    List<String> attachments = []

    def to(String recipient) {
        this.to = recipient
        return this
    }

    def from(String sender) {
        this.from = sender
        return this
    }

    def subject(String subj) {
        this.subject = subj
        return this
    }

    def body(String content) {
        this.body = content
        return this
    }

    def attach(String file) {
        this.attachments << file
        return this
    }

    def send() {
        println """
        |Sending email:
        |  From: ${from}
        |  To: ${to}
        |  Subject: ${subject}
        |  Body: ${body}
        |  Attachments: ${attachments.join(', ')}
        """.stripMargin()
    }
}

// Using the Email DSL
new EmailBuilder()
    .from('admin@example.com')
    .to('user@example.com')
    .subject('Welcome to Groovy!')
    .body('This demonstrates Groovy\'s fluent interface capabilities')
    .attach('welcome.pdf')
    .attach('guide.pdf')
    .send()

// XML Builder - Built-in Groovy DSL
def writer = new StringWriter()
def xml = new MarkupBuilder(writer)

xml.project(name: 'MyApp', version: '1.0.0') {
    dependencies {
        dependency(group: 'org.apache.groovy', name: 'groovy', version: '4.0.0')
        dependency(group: 'junit', name: 'junit', version: '4.13.2')
    }
    build {
        plugins {
            plugin(id: 'java')
            plugin(id: 'application')
        }
    }
}

println writer.toString()

// Configuration DSL
class Config {
    Map<String, Object> settings = [:]

    def database(@DelegatesTo(DatabaseConfig) Closure closure) {
        def db = new DatabaseConfig()
        closure.delegate = db
        closure.resolveStrategy = Closure.DELEGATE_FIRST
        closure()
        settings.database = db
    }

    def server(@DelegatesTo(ServerConfig) Closure closure) {
        def srv = new ServerConfig()
        closure.delegate = srv
        closure.resolveStrategy = Closure.DELEGATE_FIRST
        closure()
        settings.server = srv
    }
}

class DatabaseConfig {
    String host
    int port
    String username
    String password
    String database

    def host(String h) { this.host = h }
    def port(int p) { this.port = p }
    def username(String u) { this.username = u }
    def password(String p) { this.password = p }
    def database(String d) { this.database = d }

    String toString() {
        return "DB: ${username}@${host}:${port}/${database}"
    }
}

class ServerConfig {
    String host
    int port
    int maxConnections

    def host(String h) { this.host = h }
    def port(int p) { this.port = p }
    def maxConnections(int mc) { this.maxConnections = mc }

    String toString() {
        return "Server: ${host}:${port} (max: ${maxConnections})"
    }
}

// Using the Configuration DSL
def config = new Config()
config.with {
    database {
        host 'localhost'
        port 5432
        username 'admin'
        password 'secret'
        database 'myapp'
    }

    server {
        host '0.0.0.0'
        port 8080
        maxConnections 100
    }
}

println config.settings.database
println config.settings.server

// Gradle-like DSL example
class Project {
    String group
    String name
    String version
    List<Map> dependencies = []

    def dependencies(Closure closure) {
        closure.delegate = this
        closure.resolveStrategy = Closure.DELEGATE_FIRST
        closure()
    }

    def implementation(String notation) {
        def parts = notation.split(':')
        dependencies << [
            scope: 'implementation',
            group: parts[0],
            name: parts[1],
            version: parts.size() > 2 ? parts[2] : '+'
        ]
    }

    def testImplementation(String notation) {
        def parts = notation.split(':')
        dependencies << [
            scope: 'testImplementation',
            group: parts[0],
            name: parts[1],
            version: parts.size() > 2 ? parts[2] : '+'
        ]
    }
}

def project = new Project(
    group: 'com.example',
    name: 'my-app',
    version: '1.0.0'
)

project.dependencies {
    implementation 'org.apache.groovy:groovy:4.0.0'
    implementation 'com.google.guava:guava:31.0-jre'
    testImplementation 'junit:junit:4.13.2'
    testImplementation 'org.spockframework:spock-core:2.0-groovy-3.0'
}

println "\nProject: ${project.group}:${project.name}:${project.version}"
println "Dependencies:"
project.dependencies.each { dep ->
    println "  [${dep.scope}] ${dep.group}:${dep.name}:${dep.version}"
}

// Method missing for dynamic method dispatch
class DynamicRouter {
    Map<String, Closure> routes = [:]

    def methodMissing(String name, args) {
        if (name.startsWith('on') && args.size() == 2) {
            def method = name.substring(2).toUpperCase()
            def path = args[0]
            def handler = args[1]
            routes["${method} ${path}"] = handler
            println "Registered route: ${method} ${path}"
        } else {
            throw new MissingMethodException(name, this.class, args)
        }
    }

    def handle(String method, String path) {
        def key = "${method.toUpperCase()} ${path}"
        def handler = routes[key]
        if (handler) {
            handler()
        } else {
            println "404 Not Found: ${key}"
        }
    }
}

def router = new DynamicRouter()

// Define routes using dynamic methods
router.onGet('/users') { println "GET /users - List all users" }
router.onPost('/users') { println "POST /users - Create new user" }
router.onGet('/users/:id') { println "GET /users/:id - Get user by ID" }
router.onDelete('/users/:id') { println "DELETE /users/:id - Delete user" }

// Dispatch requests
println "\nHandling requests:"
router.handle('GET', '/users')
router.handle('POST', '/users')
router.handle('DELETE', '/users/:id')
router.handle('GET', '/unknown')
