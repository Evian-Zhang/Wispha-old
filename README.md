# Wispha

`Wispha` is a project file management tool written in Rust. In a complex project consisting of enormous files and directories, `Wispha` can produce `wispha` file efficiently to store hierarchical information and descriptions of each file. `Wispha` can also make it easier to read information of relationship between files.

Other versions:

* [简体中文](README_cn.md)

## `.wispha` format
`.wispha` format uses a hierarchical grammar to store information. A standard `LOOKME.wispha` file is given as follow:

```
+ [file path]
$ROOT_DIR/

+ [name]
wispha_test

+ [entry type]
directory

+ [description]
An example test directory to display directory type

+ [subentry]
++ [file path]
$ROOT_DIR/test1.cpp

++ [name]
test1.cpp

++ [entry type]
file

++ [description]
A .cpp file to display file type

+ [subentry]
++ [file path]
test2.rs

++ [name]
test2.rs

++ [entry type]
file

++ [description]
A .rs file to show relative file path

+ [subentry]
++ [entry file path]
$ROOT_DIR/subdir/LOOKHIM.wispha
```

`.wispha` file consists of properties with hierarchical information. Each property has a header and a body. For example:

```
+ [name]
wispha_test
```

is a property. It has header `+ [name]` and body `wispha_test`.

### Property header

A property header consists of a hierarchical marker and a category. The number of "+" sign marks the layer of current property, and the content inside bracket, i.e. `name`, is the category of this property.

The layer of a property can be any positive integer. The first property of a file must be in the first layer. Only the body of `subentry` property can be property in next layer. For example:

```
+ [subentry]
++ [file path]
$ROOT_DIR/test1.cpp

++ [name]
test1.cpp

++ [entry type]
file

++ [description]
A .cpp file to display file type
```

The first layer of file must contain:

* if contains `entry file path`, other properties will be ignored
* if not contain `entry file path`, must contain `file path`, `name`, `entry type`. `description` and `subentry` are optional.

### Property body

* The body of all properties except `description` and `subentry` should only consists of a single non-blank line, other whitespace shall be ignored.
* All characters in the body of `desciption` is valid, including whitespace.
* The body of `file path` can be absolute, relative, or start with `$ROOT_DIR`. When `Wispha` analyzes this part, the original path passed in when calling `Wispha` in commandline replaces the `$ROOT_DIR`.
* The body of `entry file path` is a path to another `.wispha` file. When `Wispha` analyzes this part, it will go to that path to analyze that file, and turn the output entry to the subentry in here. This property can only in the body of `subentry` property, or in the first layer of a file. Once the property is found, other properties in the same layer is omitted.
* The body of `entry type` can only be `directory` or `file`. This content merely marks the type in file system, the entry of type `file` can also have `subentry` property.

For portability, relative path or path which starts with `$ROOT_DIR` is recommonded when dealing with path.

## Usage

### Generate

Generally speaking, there are two ways to generate `.wispha` recording files: to generate flat or recursively. To generate flat, is to collect all files, directories in the target project into a signle `.wispha` file. And to generate recursively, is to only collect files and directories belonging to the current directory into a file, and there are more than one `.wispha` file in total. The sup directory's `.wispha` format file use `entry path file` property to refer to the sub directory's `.wispha` format file.

For example, our project's hierachical layers are given as follow:

```
- project
-- src
--- main.rs

-- test
--- test.rs

-- README.md
```

To generate flat, we have one `.wispha` format file in `project` directory, and it recores all files and directories: `src`, `src/main.rs`, `test`, `test/test.rs`, `README.md`. And to generate recursively, we have three `.wispha` format files lying in `project`, `src` and `test` directory.

For a given directory with path `path/to/directory`, we can use the command

```bash
Wispha generate -f path/to/directory
```

to generate `LOOKME.wispha` file in that directory.

We can use  `-r` option

```bash
Wispha generate -r path/to/directory
```

to generate `LOOKEME.wispha` recursively.

Generally speaking, generating recursively is more scalable, and won't lead to deep recursion when parsing. So generating recursively is recommanded, and `-r` option is made default when we generating. So we could omit `-r` option when we want to generate recursively.

If `path/to/directory` is not given, `Wispha` will generate at current path by default.

Apart from `-f` and `-r`, `generate` command supports the following options:

* `-t`<br />To specify the number of threads. 4 by default.

### Analyze

For a given directory with path `path/to/LOOKME.wispha`, we can use the command

```bash
Wispha look path/to/LOOKME.wispha
```

to analyze it. This will replace all `$ROOT_DIR` with `path/to/`.

Like `generate` command, we can use `-t` option to specify the number of threads.

If analyzing successfully, we will enter interactive mode.

```bash
$ Wispha look path/to/LOOKME.wispha
Working on looking...
Looking ready!
wispha@$ROOT_DIR/ >
```

Type `q` to quit.

#### Change current entry

```bash
wispha@some/path > cd path/to/destination
```

`Wispha` will locate `path/to/destination` via `subentry` and `name`. For example:

```
+ [subentry]
++ [file path]
$ROOT_DIR/path1

++ [name]
path2
```

When current entry is `$ROOT_DIR`, if we type `cd path1`, it will fail. If we type `cd path2`, then we can successfully change the entry.

We support using relative path or path starting with `$ROOT_DIR` in the parameter of `cd`, and using `..` to access super directory is also supported.

If we want to access entry via file path, we can add `-l` option, like:

```bash
wispha@some/path > cd -l path/to/destination
```

#### Inspect subentries

```bash
wispha@some/path > ls
```

This command can list all subentries of current entry.

Moreover, we can add a path after `ls`, which can list all subentries of the entry corresponding to the path. And the path is similar to `cd`, which could add `-l` option to force file path.

#### Inspect property

We can use `info` command to inspect a property. For example, if we want to inspect the content of `description` property of current entry, we could use command

```bash
wispha@some/path > info description
```

It is worth noting that, for property whose name contains whitespace, such as `entry type`, we need to put it inside a pair of quote signs. For example:

```bash
wispha@some/path > info "entry type"
```

### State inspect

For a given directory with path `path/to/LOOKME.wispha`, we can use the command

```bash
Wispha state path/to/LOOKME.wispha
```

to inpect all files that are in project at `path/to` and not be recorded by Wispha. The key-value pairs `ignored_files` and `allow_hidden_files` in `generate` table of `.wispha` are still valid.

Moreover, we can add `-g` option:

```bash
Wispha state -g path/to/LOOK<E.wispha
```

This command only inspect those files which are recorded by Git and not recorded by Wispha.

Like `generate` command, we can use `-t` option to specify the number of threads.

### Convert

Now we support conversion from the project recorded by `.wispha` files to JSON and TOML.

For a given directory with path `path/to/LOOKME.wispha`, we can use the command

```bash
Wispha -l JSON path/to/LOOKME.wispha -o path/to/LOOKME.json
```

to convert.

`-l` option supports `TOML` and `JSON`, and is `JSON` by default.

### Advanced usage

We can create a `.wispharc` file in the root directory of the project as configuration file. `.wispharc` file uses [TOML](https://github.com/toml-lang/toml) grammar. A common `.wispharc` file is given as follow:

```toml
[generate]
ignored_files = [
".DS_Store",
"*.wispha",
".wishparc"
]
allow_hidden_files = false

[[properties]]
name = "Author"
default_value = "Me"

[[properties]]
name = "Committer"
```

Supported key-value pairs of `generate` table are:

* `ignored_files`<br />Value is of array type. We can add file names which we want to be ignored when generating `LOOKME.wispha` file. The file name can be patterns described in [gitignore](https://git-scm.com/docs/gitignore), namely, `*.wispha` matches all file whose extension is `wispha`.
* `allow_hidden_files`<br />Value is of boolean type. If its value is `true`, then when generating `LOOKME.wispha` file, all hidden files starts with `.` is also included. This value is `false` by default.
* `wispha_name`<br />Value is of string type. Used to specify the name of `wispha` file. `LOOKME.wispha` by default.

In the array of tables `properties`, each table consists of key-value pairs `name` and `default_value`, where `default_value` is optional. If we are not satisfied with built-in properties, we can add our customized properties such as:

```
+ [Author]
Evian Zhang

+ [Committer]
Evian Zhang
```

And in interactive mode, we can use commands like `info Author` to inspect. When there is no configuration file, `Wispha` will ignore those properties.

Moreover, if a `property` table has key-value pair of `default_value`, then when generating `LOOKME.wispha` file, each entry will add the property with the given default_value.