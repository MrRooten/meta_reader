Update log:
2023.3.30

 · dump usn日志

2023.3.29

 · 新增NTFS USN Journal解析功能

2023.2.27

 · 搜索磁盘功能新增指向对应文件功能

2023.2.24:

 · 搜索磁盘功能新增正则搜索（utf8和utf16,二进制搜索）

2023.2.23:

 · 新增Ntfs列出指定目录下删除文件功能

 · Beta：指定二进制数据搜索磁盘功能

NTFS功能(处理本地磁盘需要管理员权限)
DUMP USN日志
默认会dump到当前目录的usn_log.txt，可以在后面添加'out=${path}'参数来指定文件  
PS:目前的LOG有可能不会记录在程序运行时的文件，有可能是NTFS在写Journal时造成的问题目前没弄清楚  
PS2: 根据测试，我的电脑因为需要经常编译程序会触发的日志较多，可以保存到最多前一天的记录，如果操作不频繁的电脑应该可以保存相对多天数的日志  

![image](https://user-images.githubusercontent.com/25635931/229054812-4fc45118-967e-40ed-8757-ac611ac7dc63.png)
![image](https://user-images.githubusercontent.com/25635931/229054860-12af5546-098f-4fe6-9fa4-dfb0abf64eb7.png)

搜索删除文件功能
参数:

path：指定目录
![image](https://user-images.githubusercontent.com/25635931/223934395-bc72171b-814a-4afa-8694-c750dde4e192.png)



搜索磁盘中的二进制数据
#目前还在完善，思考展示什么数据好，新增什么匹配模式，目前可以通过文件和16进制和Base64，regex，string匹配，参数之间使用逗号分隔

前面的数字是磁盘的绝对偏移量，后面的是匹配内容，之后是关联的文件，如果文件不存在则指向对应的LCN(簇)，ref_file=true参数可以关联文件，ref_file参数默认为false
![image](https://user-images.githubusercontent.com/25635931/223934859-f8575bec-b759-4b86-916d-c08e9eeded4a.png)



查看文件属性功能
用于获取文件落地时间，其中creaction2，access2，modify2所关联的事件是基于文件属性名的时间生成的，较原来的方法不易被攻击者修改，其时间基本等同于落地时间
![image](https://user-images.githubusercontent.com/25635931/223934487-5c8f193b-94e5-4426-8135-7077fb6bd1a2.png)


 

EXT4功能(处理本地磁盘需要对磁盘有可读权限)
搜索删除的文件
![image](https://user-images.githubusercontent.com/25635931/223934527-4d7549dd-fe26-4967-b95a-2255d6cf9205.png)


搜索通过日志可以恢复的文件
写入文件并删除文件

![image](https://user-images.githubusercontent.com/25635931/223934613-619329d4-e7a2-44b2-937a-ea20b38a75e7.png)

![image](https://user-images.githubusercontent.com/25635931/223934927-d789a99e-809c-4dc3-a9f3-b9a8771f47e4.png)
![image](https://user-images.githubusercontent.com/25635931/223934980-17f65bb3-dfca-4d4e-afee-c5349bf8a381.png)
![image](https://user-images.githubusercontent.com/25635931/223935021-f2f46077-aa92-4df5-9384-b7041325a936.png)
![image](https://user-images.githubusercontent.com/25635931/223935053-4441ea89-4621-422f-9230-0986408d1db7.png)
![image](https://user-images.githubusercontent.com/25635931/223935086-5c61760a-777a-4b02-828e-cd54c49bdc2c.png)

最后一个不同的原因是因为文件被删除了，inode被占用导致指向另一个引用的文件，这里指向的是，该项目编译所使用的源文件

