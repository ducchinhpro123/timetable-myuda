# A simple program fetching timetable from my.uda.edu.vn

Because I'm too lazy to open the browser just to check what I'm going to study today.

## Usage

```Usage: request <USERNAME> <PASSWORD>```

### Example

[duchinh@99562 release]$ ./request Your_id password
Fetching login page...
Fetching timetable page...
  ____    _  _        __  _    ___       __  ____     ___    ____    _  _   
 |___ \  | || |      / / / |  / _ \     / / |___ \   / _ \  |___ \  | || |  
   __) | | || |_    / /  | | | | | |   / /    __) | | | | |   __) | | || |_ 
  / __/  |__   _|  / /   | | | |_| |  / /    / __/  | |_| |  / __/  |__   _|
 |_____|    |_|   /_/    |_|  \___/  /_/    |_____|  \___/  |_____|    |_|  
                                                                            

+-----+-------+------+------------------------------+-------------------------------+---------------------------------+-------------------------------+
| THỨ | BUỔI  | TIẾT | PHÒNG                        | HỌC PHẦN                      | GIẢNG VIÊN                      | LỚP HỌC TẬP                   |
+-----+-------+------+------------------------------+-------------------------------+---------------------------------+-------------------------------+
| 2   | Sáng  | 1-3  | Online        -  Link online | Quản lý dự án (2tc)           | ThS. ABC (GVCH)                 | 4848(ST22A,ST22B,ST22C)       |
+-----+-------+------+------------------------------+-------------------------------+---------------------------------+-------------------------------+
| 3   | Chiều | 1-3  | 999                          | Lập trình Web 1 (3tc)         | ThS. ABC (GVCH)                 |                               |
+-----+-------+------+------------------------------+-------------------------------+---------------------------------+-------------------------------+
| 3   | Chiều | 4-6  | 999                          | Lập trình di động 1 (3tc)     | ThS. ABC (GVCH)                 |                               |
+-----+-------+------+------------------------------+-------------------------------+---------------------------------+-------------------------------+
| 4   | Chiều | 1-3  | 999                          | Hệ điều hành (3tc)            | TS. ABC (GVTG)                  |                               |
+-----+-------+------+------------------------------+-------------------------------+---------------------------------+-------------------------------+
| 5   | Chiều | 1-3  | Tiên Sơn                     | Giáo dục thể chất 3 (1tc)     | ThS. ABC(TC) (GVCH)             | 1858(SE22A,ST22A,ST22B,CO22A) |
+-----+-------+------+------------------------------+-------------------------------+---------------------------------+-------------------------------+
| 5   | Chiều | 4-4  | Tiên Sơn                     | Giáo dục thể chất 3 (1tc)     | ThS. ABC(TC) (GVCH)             | 1858(SE22A,ST22A,ST22B,CO22A) |
+-----+-------+------+------------------------------+-------------------------------+---------------------------------+-------------------------------+
| 6   | Sáng  | 1-3  | 999                          | Trí tuệ nhân tạo (3tc)        | TS. ABC (GVTG)                  |                               |
+-----+-------+------+------------------------------+-------------------------------+---------------------------------+-------------------------------+
| 6   | Sáng  | 4-6  | 999                          | Kiểm thử phần mềm 1 (3tc)     | TS. ABC (GVTG)                  |                               |
+-----+-------+------+------------------------------+-------------------------------+---------------------------------+-------------------------------+
| 6   | Chiều | 1-3  | 999                          | Lập trình mã nguồn mở 1 (3tc) | ĐH. ABC (GVTG)                  |                               |
+-----+-------+------+------------------------------+-------------------------------+---------------------------------+-------------------------------+
| 7   | Sáng  | 4-6  | 999                          | Lập trình mã nguồn mở 1 (3tc) | ĐH. ABC (GVTG)                  |                               |
+-----+-------+------+------------------------------+-------------------------------+---------------------------------+-------------------------------+

