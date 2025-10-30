rust 实现一个 根据配置文件来选择压缩当前目录的格式，忽略哪些目录文件（采用gitingore那样，但建议放在配置文件里），默认打包当前目录，把当前目录名作为压缩包名字, 配置文件里指定后缀，自动选择不同的库来打包，目前只实现.zip .tar.gz .7z 
命令行显示进度，添加init命令创建ztr.toml,添加一个show显示可以打包的类型
添加新的解析，ingore和ingore_file存在一个即可，都存在则ingore优先级最高，而且ingore_file是指定默认路径，可以弄个注释ignore_file="./.gitignore"