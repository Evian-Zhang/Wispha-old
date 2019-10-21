# Wispha

Wispha是一个使用Rust语言编写的项目文件管理工具。面对复杂的项目文件结构，Wispha可以高效地生成`LOOKME.wispha`来存储文件之间的结构层次和每个文件的描述，也可以通过读取项目的`LOOKME.wispha`文件展示项目的文件层次及相关信息。

## `.wispha`格式

`.wispha`文件采用层次化的文法记录文件信息。一个标准的`LOOKME.wispha`文件内容如下：

```
+ [absolute path]
$ROOT_DIR/

+ [name]
wispha_test

+ [entry type]
directory

+ [description]
An example test directory to display directory type

+ [subentry]
++ [absolute path]
$ROOT_DIR/test1.cpp

++ [name]
test1.cpp

++ [entry type]
file

++ [description]
A .cpp file to display file type

+ [subentry]
++ [absolute path]
test2.rs

++ [name]
test2.rs

++ [entry type]
file

++ [description]
A .rs file to show relative absolute path

+ [subentry]
++ [entry file path]
$ROOT_DIR/zjd/LOOKME.wispha
```

`.wispha`文件由层次化的属性组成。每个属性有头和内容。如

```
+ [name]
wispha_test
```

是一个属性，其头为`+ [name]`, 内容为`wispha_test`.

### 属性头

一个属性的头由层次标记和类别组成，`+`号的数目标记了当前属性所在的层次，中括号内的内容`name`是该属性的类别。

一个属性的层次可以是任意正整数。一个文件的第一个属性必须是处在第一层。只有`subentry`属性的内容可以是下一层的属性。如

```
+ [subentry]
++ [absolute path]
$ROOT_DIR/test1.cpp

++ [name]
test1.cpp

++ [entry type]
file

++ [description]
A .cpp file to display file type
```

### 属性内容

* 除了`description`和`subentry`以外，其他属性的内容都只允许出现一行非空白行。其余空白字符将被忽略。
* 对于`desciption`属性，其内容的所有字符都有效。
* 对于`path`属性，其内容可以是绝对路径，相对路径，或是以`$ROOT_DIR`开头的路径。当`Wispha`程序分析到该文件时，会以最初调用该指令时传入的路径作为`$ROOT_DIR`.
* 对于`entry file path`属性，其内容为另一个`.wispha`文件的路径。当`Wispha`程序分析到这个属性时，会取指定路径分析那个文件作为该属性对应的文件。该属性只能出现在`subentry`属性的内容中或者文件的第一层属性中。一旦出现，则其他同层次的属性均被忽略。
* 对于`entry type`属性，其内容只可以为`directory`或`file`. 这个内容只是标记其在文件系统中的事实情况，`file`类型的主体依然可以有`subentry`.

## 使用方法

### 生成

对于指定目录，其路径为`path/to/directory`, 可使用命令

```bash
Wispha generate path/to/directory
```

在指定目录下生成`LOOKME.wispha`文件。

值得注意的是，这里只在指定目录下生成一个文件`path/to/directory/LOOKME.wispha`, 该目录的所有文件以`subentry`的形式记录在其中。可以在命令中加入`-r`选项

```bash
Wispha generate -r path/to/directory
```

以递归地生成`LOOKME.wispha`文件。即在指定目录及其所有子目录中均生成`LOOKME.wispha`文件，每个子目录在其上层目录的`LOOKME.wispha`中均只以`entry file path`属性出现。

### 分析

对于指定的`.wispha`文件，其路径为`path/to/LOOKME.wispha`, 可使用命令

```bash
Wispha look path/to/LOOKME.wispha
```

指令进行分析。其会将`path/to/`路径作为所有相关文件中的`$ROOT_DIR`.

若分析成功，则进入交互模式。

```bash
$ Wispha look path/to/LOOKME.wispha
Working on looking...
Looking ready!
wispha@$ROOT_DIR/ >
```

输入`q`退出交互模式。

#### 切换当前主体

```bash
wispha@some/path > cd path/to/destination
```

其中`path/to/destination`默认按`subentry`和`name`属性寻找。即

```
+ [subentry]
++ [absolute path]
$ROOT_DIR/path1

++ [name]
path2
```

在当前主体为`$ROOT_DIR`时，如果输入`cd path1`会失败，输入`cd path2`会切换到对应的文件主体。

目前支持`$ROOT_DIR`开头，或相对路径，也支持在路径中加入`..`访问上层文件。

如果想要根据绝对路径切换主体，则可加入`-l`选项，即

```
wispha@some/path > cd -l path/to/destination
```

则会按绝对路径访问。

#### 查看子文件

```bash
wispha@some/path > ls
```

会显示当前主体的子主体。

此外，`ls`后还可以加入路径，即查看对应路径下的子主体。关于路径的要求和`cd`类似，也可以加入`-l`选项强制本地路径。