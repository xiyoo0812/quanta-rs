

print("hello world")

local x = 1
local y = 2

print(x, y, x+y)

local a = {a = 1, b =2} 
for k, v in pairs(a) do
	print(k, v)
end

local json = require "ljson"
local e = json.encode(a)

print("json encode:", e)

local d = json.decode(e)
for k, v in pairs(d) do
	print("json decode:", k, v)
end

function test()
	local stdfs = require "lstdfs"
	local dirs = stdfs.dir("./")
	print("test func call")
	for i, info in ipairs(dirs) do
		print(string.format("dir %d: {name = %s, type = %s} ", i, info.name, info.type))
	end
	local cur = stdfs.current_path()
	print ("current_path:", cur)
	local pcur = stdfs.parent_path(cur)
	print ("parent_path:", pcur)
	local ok, err = stdfs.chdir("../")
	print ("chdir:", ok, err)
	local cur = stdfs.current_path()
	print ("current_path:", cur)
	local ok, err = stdfs.chdir("bin")
	print ("chdir:", ok, err)
	local cur = stdfs.current_path()
	print ("current_path:", cur)
	local tdir = stdfs.temp_dir()
	print ("temp_dir:", tdir)
	local appdir = stdfs.append(tdir, "test")
	print ("appdir:", appdir)
	local concatdir = stdfs.concat(appdir, "123.png")
	print ("concatdir:", concatdir)
	local fstem = stdfs.stem("./bb/xx.lua")
	print ("file stem:", fstem)
	local filename = stdfs.filename("./bb/xx.lua")
	print ("file name:", filename)
	local ftype = stdfs.filetype("./aa.lua")
	print ("file type:", ftype)
	local fsize = stdfs.file_size("./aa.lua")
	print ("file size:", fsize)
	local ext = stdfs.extension("./aa.lua")
	print ("file extension:", ext)
	local write_time = stdfs.last_write_time("./aa.lua")
	print ("file last_write_time:", write_time)
	local isdir = stdfs.is_directory("./bb")
	print ("is dir:", isdir)
	local absolute = stdfs.absolute("./../")
	print ("file absolute:", absolute)
	local is_absolute1 = stdfs.is_absolute(cur)
	local is_absolute2 = stdfs.is_absolute("./bb")
	print ("is absolute:", is_absolute1, is_absolute2)
	local isexists1 = stdfs.exists("./bb/xx.lua")
	local isexists2 = stdfs.exists("./bb/xx1.lua")
	print ("is exists:", isexists1, isexists2)
	local ok, err = stdfs.mkdir("./bb/cc/dd")
	print ("mkdir:", ok, err)
	local ok, err = stdfs.copy_file("./bb/xx.lua", "./bb/xx1.lua")
	print ("copy:", ok, err)
	local ok, err = stdfs.rename("./bb/xx1.lua", "./bb/xx2.lua")
	print ("rename:", ok, err)
	local ok, err = stdfs.remove("./bb/xx2.lua")
	print ("remove:", ok, err)
	local ok, err = stdfs.copy_file("./bb/xx.lua", "./bb/cc/dd/xx1.lua")
	print ("copy:", ok, err)
	local ok, err = stdfs.copy("./bb/cc", "./bb/dd")
	print ("copy dir:", ok, err)
	local ok, err = stdfs.remove("./bb/cc/dd", true)
	print ("remove dir:", ok, err)
	local ok, err = stdfs.remove("./bb/dd", true)
	print ("remove dir:", ok, err)
	local paths = stdfs.split("./bb/cc/dd")
	for i, info in ipairs(paths) do
		print(string.format("path %d: %s ", i, info))
	end
	print ("remove dir:", ok, err)
	local mpp = stdfs.make_preferred(".\\Users\\example\\path\\to\\file.txt")
	print ("make_preferred:", mpp)
	local rname = stdfs.root_name(".\\Users\\example\\path\\to\\file.txt")
	print ("root_name:", rname)
	local rpath = stdfs.root_path(".\\Users\\example\\path\\to\\file.txt")
	print ("root_path:", rpath)
	local relative_path = stdfs.relative_path(".\\Users\\example\\path\\to\\file.txt")
	print ("relative_path:", relative_path)
end

function test1(a, b)
	print("test1 func call, args: ", a, b)
	return a, a + b, "ABC"
end
