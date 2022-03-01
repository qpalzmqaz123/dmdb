# Dmdb

达梦数据库 rust 驱动

## 依赖

该驱动使用达梦官方提供的 DPI 接口，编译和运行均需 libdmdpi.so

## 支持

### 数据类型

#### 字符数据类型

- [x] CHAR
- [x] CHARACTER
- [x] VARCHAR
- [x] VARCHAR2

#### 数值数据类型

- [x] NUMERIC
- [x] DECIMAL
- [x] DEC
- [x] NUMBER
- [x] INTEGER
- [x] INT
- [x] BIGINT
- [x] TINYINT
- [x] BYTE
- [x] SMALLINT
- [ ] BINARY
- [ ] VARBINARY
- [x] FLOAT
- [x] DOUBLE
- [x] REAL
- [x] DOUBLE PRECISION

#### 位串数据类型

- [x] BIT

#### 日期时间数据类型

- [ ] DATE
- [ ] TIME
- [x] TIMESTAMP/DATETIME
- [ ] INTERVAL YEAR TO MONTH
- [ ] INTERVAL YEAR
- [ ] INTERVAL MONTH
- [ ] INTERVAL DAY
- [ ] INTERVAL DAY TO HOUR
- [ ] INTERVAL DAY TO MINUTE
- [ ] INTERVAL DAY TO SECOND
- [ ] INTERVAL HOUR
- [ ] INTERVAL HOUR TO MINUTE
- [ ] INTERVAL HOUR TO SECOND
- [ ] INTERVAL MINUTE
- [ ] INTERVAL MINUTE TO SECOND
- [ ] INTERVAL SECOND
- [ ] TIME WITH TIME ZONE
- [ ] TIMESTAMP WITH TIME ZONE
- [ ] TIMESTAMP WITH LOCAL TIME ZONE

#### 多媒体数据类型

- [x] TEXT
- [x] LONG
- [x] LONGVARCHAR
- [ ] IMAGE
- [ ] LONGVARBINARY
- [ ] BLOB
- [x] CLOB
- [ ] BFILE

