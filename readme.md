# async memcached client

wip

more doc see:

https://docs.rs/memcached/0.2.0/memcached/struct.Client.html

## warning

1. 除了 increment， decrement 别的函数不能传入数字
1. 对于数字 get 的时候要用字符串手动 trim
1. stats 还不能用
1. version 也有点问题
