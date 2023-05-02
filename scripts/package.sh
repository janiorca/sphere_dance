BASEDIR=$(dirname "$0")
$BASEDIR/build.sh
./tools/Crinkler/releases/crinkler23/Win64/Crinkler.exe /OUT:$BASEDIR/../target/release/namekusei.exe /SUBSYSTEM:WINDOWS ./target/i686-pc-windows-msvc/release/deps/namekusei.o /ENTRY:mainCRTStartup "/LIBPATH:C:\Program Files (x86)\Windows Kits\10\Lib\10.0.18362.0\um\x86" gdi32.lib user32.lib opengl32.lib kernel32.lib winmm.lib
