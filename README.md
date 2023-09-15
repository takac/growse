# growse

A cli tool to open the GitHub, Bitbucket or Gitlab page for repos and files in
repos.

Open a repo in the browser, automatically manage if it's github, bitbucket or gitlab.
```
growse
```

Open the README.md file at line 10 in the browser on the default remote branch.
```
growse README.md
```


Open the README.md file at line 10 in the default browser on the master branch.
```
growse -b master README.md:10
```

Using the remote of `takac` open the README.md file at line 10 in the default browser on the master branch.
```
growse -r takac -b master README.md:10
```

Files can are handled relatively, so doesn't matter where you are in the
repo directory.

```
cd src
growse ../README.md
```

To use inside vim for example to open the current file at line 10 in the browser.
```
:!growse %:10
```
or open the current line which requires a bit more vimscript: 
```
:call system('growse ' . expand("%") . ":" . line('.'))
```

And can be used as a vim command, to quickly run this vimscript snippet.
```
:command Growse :call system('growse ' . expand("%") . ":" . line('.'))
```
```
:Growse
```


# Installation

Install to the cargo path in `~/.cargo/bin`
```
cargo install --path .
```

# Current Support

Currently only "extra" features are supported for GitHub, the other backends
need to be implemented.

There are some additional complications to overcome on custom urls for
backends, which will probably need to be solved via a config file. e.g.
`https://mycustomgitlab.com/myorg/myrepo/myproject/` will need to be correctly
handled.

| Backend        | Repo     | Branch    | File path   | File path with Line No.    | File path with line range |
| -------------- | -------- | --------- | ----------- | -------------------------- | --------------------      |
| GitHub         | ✅ Yes   | ✅ Yes    | ✅ Yes      | ✅ Yes                     | ❌ No                     |
| Bitbucket      | ✅  Yes  | ✅ Yes    | ❌ No       | ❌ No                      | ❌ No                     |
| Gitlab         | ✅ Yes   | ❌ No     | ❌ No       | ❌ No                      | ❌ No                     |
| Others         | ❌ No    | ❌ No     | ❌ No       | ❌ No                      | ❌ No                     |



