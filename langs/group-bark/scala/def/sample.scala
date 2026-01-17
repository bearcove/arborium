object Main extends App {
  case class Person(name: String, age: Int)

  val people = List(
    Person("Alice", 30),
    Person("Bob", 25)
  )

  people
    .filter(_.age > 20)
    .map(p => s"Hello, ${p.name}!")
    .foreach(println)
}
