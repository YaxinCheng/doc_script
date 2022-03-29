# Language Tutorial

DocScript was designed to be a simple language resemble to any other common programming languages you have used. Therefore, it focuses on representing data and layout using simple terminologies and operators. At the current stage, many language features are incomplete, but the basic framework is set up. From this documentation, you can grasp the philosophy behind the design and get your hands on the basic language features quickly.

## Constant

As the basic element of the language, constants are common used to represent a piece of data. Like many other languages, constants are statically evaluated during the compilation, so no dynamic memory allocation/deallocation is involved. This is to say constant values are fast.

### Declaration

Constant values can be declared easily, and here is an example

```c
const STRING = "Hello World" // string constant
```

From this example, we can see constants are declared with `const` keyword followed with the constant name. It is recommended to use the SCREAMING_SNAKE_CASE to name constants, but it is not mandatory.

This example above defines a constant associated with a string value, `"Hello World"`. So in the future, whenever `STRING` is used, its value is always `"Hello World"`. 

Similarly, you can define constants with other values

```c
const INT = 42 // integer constant
const FLOAT = 3.14 // float constant
const BOOL = true // boolean constant
const VOID = () // void constant
```

### Types

You may have noticed, type annotation is not part of the constant declarations. That is because DocScript uses type induction to detect the type from values directly. However, this does not mean type is insignificant in DocScript.

Types in DocScript are served as constraints to help format the documents, also it increases the readability and help writers maintain their scripts.

Similar to other languages, DocScript provides a list of primitive types: `String`, `Int`, `Float`, `Bool`, and `Void`, that can be used out of the box. Also, writers can define their own types, such as structs or traits, to help them model their document. 

There will be a more detailed discussion about types in DocScript in later sections, but for now, just remember types are not part of constant declarations, but they still exist.

 ## Struct

As mentioned before, writers can define their own types. These types are called structs. 

### Declaration

Here is an example of defining a type for an essay:

```c#
struct Essay(text: String, author: String = "Unknown") {
  const content = self.text
}
```

Through this example, you can see that custom types are created using `struct` keyword followed by the type name. Preferably, the type name should be in CamelCase, but again, it is not mandatory. 

This `Essay` type has two required fields, `author` ans `text`. From the code we know that they are both `String` types. The type annotations are required in struct declaration. The `author` field also has a default value, so if no string is provided for the `author` field during the construction, it will use `"Unknow"` as the author name. 

After the fields, there is a block which is referred as the struct body. It is an area that attributes can be defined. An attribute is a constant, and the value for the constant can dependent on the field values. For this example, the `content` is actually depending on the `text` field. The `self` here is referring to the `Essay` data itself, and we will discuss more on the syntax of data accessing. 

### Construction

Once you have defined your own type, you can create a piece of data with this type. Here is an example:

```c#
struct Essay(text: String, author: String = "Unknown") {
  const content = self.text
}

const ZEN_OF_PYTHON = Essay("Beautiful is better than ugly...", "Tim Peters")
```

This piece of code creates a piece of data with `Essay` type, and the `text` is `"Beautiful is better than ugly..."` and `author` is `"Tim Peters"`. The order of values passed in decides the values of the fields. However, writers can choose their own order if they can specify the field name in front of the values passed in:

```c
const ZEN_OF_PYTHON = Essay(author: "Tim Peters", text: "Beautiful is better than ugly...")
```

This does the same thing as the previous one, but this time the meaning of each value becomes more clear. 

Now, if we want to record an essay without knowing who the author is, we can just create the essay with author being `"Unknown"`, like this

```c#
const SOME_ESSAY = Essay("2B or not 2B...", "Unknown")
```

However, we have already given the type `Essay` the power to set the author to `"Unkown"` as a default option if we do not provide anything. In other words, the following code does the same thing as the snippet above:

```c
const SOME_ESSAY = Essay("2B or not 2B...")
```

### Access

So far we have learned how to create our own types and how to store data as one of the types we created, now is the time to learn how we can access stored data.

As DocScript is designed to be as simple as possible, so the access will be the same as many other languages: using a `.`(dot) to access data inside a congregated type (like structs).

There is no visibility limitation or other rationale, any data can be accessed and can only be accessed using dots. Here is an example:

```c
const SOME_ESSAY = Essay("2B or not 2B...")
const AUTHOR = SOME_ESSAY.author
const TEXT = SOME_ESSAY.text
const CONTENT = SOME_ESSAY.content
```

This code above will create a new constant `AUTHOR` and its value will be the same as the `author` of `SOME_ESSAY`, which is `"Unknown"` in this case. Similarly, `TEXT` will have the value of `text` of `SOME_ESSAY`. 

Dot is also used to access attributes. In this case, `content` of `SOME_ESSAY` is accessed and its value is assigned to constant `CONTENT`. 

Let's look back at the definition of `Essay`:

```c#
struct Essay(text: String, author: String = "Unknown") {
  const content = self.text
}
```

The `self` is a way of accessing the data itself from inside. It also uses dot to access data of itself. This is why attribute can have values that depend on fields or other attributes.

## Trait

Trait is a way to limit or abstract a type as a type with certain features. It was introduced to introduce duck type system into DocScript. Like `interface` in Java or `trait` in Rust, trait in DocScript is a bound or a restriction to describe a type with given values.

### Declaration

Here is an example on how to define a trait:

```scala
trait Document(title: String, content: String)
```

This trait defines any type with a string `title` and a `string` content is a document when needed. Therefore, following structs all follow the `Document` trait

```c#
struct Essay(title: String, content: String, author: String)
struct Draft(content: String) {
  const title = "Draft"
}
struct EmptyPaper {
  const content = ""
  const title = "Empty Paper"
}
```

Not just structs, traits can follow other traits too!

```scala
trait Publication(title: String, content: String, publish_year: Int)
```

The trait above follows `Document` as well.

### Derivation

Traits are automatically followed for qualified types, so there is no need to manually declared that a struct or a trait follows specific traits. This is to say, the following code is valid:

```c#
struct Library(oldest_doc: Document)
struct Essay(title: String, content: String, author: String)

const lib = Library(Essay("title", "", "Author"))
```

## Type System

Until this point, we have learned about defining struct as concrete types to store our data and defining traits as abstractions of all kind of types. However, we have not yet discussed the type system in DocScript, and here we are.

### Goal

Types in DocScript are meant to restrict the kinds of data that can be used in different structs. This restriction provides guarantees on member data accessing, it also aids the readability as it provides more details to the maintainer. 

To ease the restriction but also maintain required constraints, DocScript introduced `trait`. Based on the abstract type, the type system should be more expressive while still provide some guarantees on data accessing.

### Duck Type System

As the name was mentioned before, a "duck type system" is from a famous quote:

>  if it looks like a duck, if it walks like a duck, if it quacks like a duck, it is a duck. 

To illustrate the quote in scenarios of DocScript, if a type (`struct` or `trait`) has all the required members of a `trait`, then it is the `trait`. Based on this philosophy, DocScript was designed to perform the auto-trait-implementation when passing data to a place where type is required.

## Modules and Imports

When the document project gets bigger, it is hard to manage all the pieces in the same file. Thus, module is an indispensable part of DocScript that allows you to distribute pieces into different modules and manage them separately. Data in different files but in the same module can be used seamlessly, and import declarations is needed to use data across different modules.

### Implicit modules

Modules are implicit in DocScript. This is an attempt to make the language simpler. By 'implicit', it means there is no explict way to declare a module, but it depends on the file system of the source files. 

For example, a file `Main.ds` whose full path name is `src/Main.ds`, then all the declared types and constants will be put into the global module. A source file at `src/snacks/candies/Taffy.ds` adds all its types and constants to module `snacks.candies`. Finally, another source file at `src/snacks/candies/Lollipop.ds` can use types and constants in `Taffy.ds` (same module) and `Main.ds`(global module).

### Import declarations

Sometimes, a piece of data is needed across modules, then the pieces need to be imported to the current module. There are three ways of importing

#### Simple import

The first one imports constants or types from a designated module, with following syntax:

```rust
use snacks.candies.Taffy // import Taffy
const taffy = Taffy()
```
Or multiple of them can be imported at once
```rust
use snacks.candies.{Taffy, Lollipop} // import both Taffy and Lollipop

const taffy = Taffy()
const lollipop = Lollipop()
```

#### Module import

Instead of importing every single one of the data needed, one can import a whole module as one entity, then access data inside the module

```rust
use snacks.candies

const taffy = candies.Taffy()
const lollipop = candies.Lollipop()
```

Or maybe we just want to import `snacks` module, which makes the code like this

```rust
use snacks

const taffy = snacks.candies.Taffy()
const lollipop = snacks.candies.Lollipop()
```

#### Wildcard import

Lastly, DocScript provides a way to import whatever is used in a module

```rust
use snacks.candies.*

const taffy = Taffy()
```

In the code above, only `Taffy` is imported from the `candies` module. Other unused types and constants are not imported.

### Warnings

Please note, importing from data with the same name from different modules is not supported.

Also, when data with a specific name is imported with simple import then wildcard imports will not import any other data with the same name into the current module.

Similarly, if a piece of data with name `x` is used, but it can possibly be imported from two different modules, then DocScript will get confused and panic about this ambiguity.

## Value data and copying

As you may have noticed, DocScript has only constant data throughout the system. This means there is no way users can modify an already created data, and also data is copied when a new constant points to an existing constant.

During the project growing larger, the number of constants will grow as well. It may potentially become hard to manage or think about a name for a constant. Therefore, DocScript supports **shadowing** to ease this issue. Basically, you can declare a new constant with a name that appeared before to shadow that constant. Any code after this declaration will use this new constant instead.

Because everything is constant, structured data cannot be changed once created. Nevertheless, it is a common scenario that some changes are needed. In DocScript, it is recommended to create a new data with most of the fields copied over. Due to the rather complex syntax for creating some large structs and modifying often happens at little places, DocScript provides an easy syntax sugar for it:

```rust
struct Essay(title: String, author: String)

// original
const UNKNOWN_ESSAY = Essay("title", "UNKNOWN")
// new
const KNOWN_ESSAY = UNKNOWN_ESSAY.author("author")
```

The second constant declaration creates a new `Essay` with the same title as `UNKNOWN_ESSAY` but with `author` changed to `"author"`. Again, it does nothing to the original `UNKNOWN_ESSAY`, this is just an easy way to copy a piece of data while changing it slightly, after creation, the data becomes immutable. 

More conveniently, this syntax can be chained to create a copy with multiple fields modified. See this example

```rust
struct Publication(
  title: String,
  author: String,
  publish_year: Int,
  language: String
)

const ANNE_OF_GREEN_GABLES = Publication(
  title: "Anne of Green Gables",
  author: "Lucy Maud Montgomery",
  publish_year: 1908,
)

const THE_LITTLE_PRINCE = ANNE_OF_GREEN_GABLES
														.title("The Little Prince")
														.publish_year(1943)
														.author("Antoine de Saint-Exupéry")
```

This will copy the `ANNE_OF_GREEN_GABLES` while changing the `title` to `"The Little Prince"`, `publish_year` to `1943`, and changing `author` to `"Antoine de Saint-Exupéry"`.

With this feature combined with shadowing, it should be powerful enough to drive daily document works.

## Conclusion

Overall, this is the end of this short language tutorial. As more features are coming, this tutorial will be updated accordingly. Hope you enjoy building up your documents with this small and simple language!
