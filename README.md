::⚠️ ntfs usn dump has some bugs, maybe the data run problem



NTFS(Need Administrator)
suport functions:  
- stat
- deleted_files
- carve_usn

Search deleted files  
Argument:  
- path：directory  
```shell
.\meta_reader.exe ntfs --function deleted_files -d \\.\C: --options "path=C:\"
count: 15
1723627 Some("Windows.old") 2
```

Search binary data in disk  
```shell
.\meta_reader.exe ntfs -f search_disk -d \\.\C: -o encode=regex,to_search=123
```



File stat  
```shell
 .\meta_reader.exe ntfs --function stat -d \\.\C: --options "path=C:\Windows"
filename: Some("Windows")
        index: 1724114
        fullpath: Some("Windows")
        creation: Some(2022-05-07T13:17:22+08:00)
        access: Some(2024-12-23T13:49:45+08:00)
        modify: Some(2024-12-19T11:29:01+08:00)
        creation real(from filename): Some(2022-05-07T13:17:22+08:00)
        stream list: None
```

EXT4(Need privilege to read disk file. like /dev/sdb)  
Support function  
- list_deleted_files
- journal_recover_file
- list_files
- read_file
- list_recoverable
- search_deleted_files
- search_recoverable_files

Search deleted files  
![image](https://user-images.githubusercontent.com/25635931/223934527-4d7549dd-fe26-4967-b95a-2255d6cf9205.png)  

Search recoverable files by jbd2  

![image](https://user-images.githubusercontent.com/25635931/223934613-619329d4-e7a2-44b2-937a-ea20b38a75e7.png)  

![image](https://user-images.githubusercontent.com/25635931/223934927-d789a99e-809c-4dc3-a9f3-b9a8771f47e4.png)  
![image](https://user-images.githubusercontent.com/25635931/223934980-17f65bb3-dfca-4d4e-afee-c5349bf8a381.png)  
![image](https://user-images.githubusercontent.com/25635931/223935021-f2f46077-aa92-4df5-9384-b7041325a936.png)  
![image](https://user-images.githubusercontent.com/25635931/223935053-4441ea89-4621-422f-9230-0986408d1db7.png)  


