# PHPlus

A better way to write PHP.

PHPlus is a transpiled programming language that compiles `.ez` source files into modern PHP. It provides cleaner syntax, improved OOP ergonomics, optional strict typing, and static validation before PHP execution.

---

## USAGE

### Compile a Single File

```bash
./phplus my_program.ez
```

Output:

```text
my_program.ez -> my_program.php
```

### Compile an Entire Directory

```bash
./phplus --dir src
```

or

```bash
./phplus -d src
```

All `.ez` files will be recursively compiled into PHP.

### Disable Strict Mode

```bash
./phplus --no-strict my_program.ez
```

or

```bash
./phplus -ns my_program.ez
```

In strict mode (default), all variables, parameters, and function return values must have explicit type annotations.

---

## EXAMPLE

### PHPlus

```ez
let name: string = "World"

fn greet(person: string): void {
    print "Hello, " + person
}

greet(name)
```

### Generated PHP

```php
<?php
declare(strict_types=1);

$name = (string) "World";

function greet(string $person): void {
    echo "Hello, " + $person;
}

greet($name);
```

---

## FEATURES

### Cleaner Syntax

```ez
let x = 10
print x
```

instead of

```php
$x = 10;
echo $x;
```

### Better OOP

```ez
user.name
user.login()
```

transpiles to

```php
$user->name;
$user->login();
```


### Functions

```ez
fn add(a: int, b: int): int {
    return a + b
}
```

### Classes

```ez
class User {
    let name: string = ""

    fn greet(): void {
        print self.name
    }
}
```

### Control Flow

```ez
if (x > 5) {
    print "large"
}

while (running) {
    doSomething()
}

for (let i = 0; i < 10; i = i + 1) {
    print i
}
```

### Arrays

```ez
let items: array = [1, 2, 3]
print items[0]
```

### Includes

```ez
include "utils.ez"
```

Automatically becomes:

```php
require_once "utils.php";
```

---

## TYPE SYSTEM

PHPlus supports:

* `int`
* `float`
* `string`
* `bool`
* `array`
* `void`
* `null`
* `mixed`
* `never`
* `object`
* `callable`

### Nullable Types

```ez
let user: ?User = null
```

### Union Types

```ez
let value: int|string = 42
```

### Strict Mode Validation

PHPlus performs compile-time checks for:

* Missing type annotations
* Invalid assignments
* Class existence validation
* Type compatibility checks
* Invalid object construction

Example:

```ez
let age: int = "hello"
```

Produces:

```text
Type mismatch: cannot assign string to variable 'age' declared as int
```

before PHP is generated.

---

## CURRENT FEATURES

* Variables (`let`)
* Printing (`print`)
* Functions
* Classes
* Methods
* Properties
* Access modifiers (`private`)
* Object creation (`new`)
* If / Else
* While loops
* For loops
* Arrays
* Superglobals

  * `$_POST`
  * `$_GET`
  * `$_SESSION`
  * `$_SERVER`
  * `$_COOKIE`
* Type annotations
* Nullable types
* Union types
* Compile-time type checking
* Recursive project compilation
* Include system

---

## FUTURE

Planned features include:

* Generics
* Interfaces
* Enums
* Match expressions
* Namespaces
