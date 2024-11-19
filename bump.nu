def new_versions [version] {
ls packages/ | get name | each {|it| $it | path parse | get stem} | each {|it| { name: $it, version: (cargo search $it --limit=1 --color=never | awk '/(.*) = "([0-9]+(\.)?)+-([a-zA-Z])*.([0-9]+(\.)?)+"/ {gsub(/"/,""); print($3)}')}}
}
