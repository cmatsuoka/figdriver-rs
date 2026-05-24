# A FIGdriver implementation in Rust

Warning: work in progress. Names and API will change. 

A FIGdriver is a program that uses FIGlet FIGfonts to create banners in
ASCII-art style. This is a FIGdriver implementation written as a small
project to learn Rust and exercise string manipulation.

## Differences compared to FIGlet

|                               | FIGlet       | Our FIGdriver        |
| ---                           | ---          | ---                  |
| Terminal width                | Set to _n_-1 | Set to _n_           |
| Non-UTF8 sub-characters       | Display      | Discard FIGcharacter |
| Word spacing in overlap mode  | No           | Yes                  |
| End space in paragraph mode   | Yes          | No                   |
| Right-to-left support         | Yes          | No                   |
| Control files                 | Yes          | No                   |
