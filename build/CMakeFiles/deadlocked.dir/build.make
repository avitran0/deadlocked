# CMAKE generated file: DO NOT EDIT!
# Generated by "Unix Makefiles" Generator, CMake Version 3.30

# Delete rule output on recipe failure.
.DELETE_ON_ERROR:

#=============================================================================
# Special targets provided by cmake.

# Disable implicit rules so canonical targets will work.
.SUFFIXES:

# Disable VCS-based implicit rules.
% : %,v

# Disable VCS-based implicit rules.
% : RCS/%

# Disable VCS-based implicit rules.
% : RCS/%,v

# Disable VCS-based implicit rules.
% : SCCS/s.%

# Disable VCS-based implicit rules.
% : s.%

.SUFFIXES: .hpux_make_needs_suffix_list

# Command-line flag to silence nested $(MAKE).
$(VERBOSE)MAKESILENT = -s

#Suppress display of executed commands.
$(VERBOSE).SILENT:

# A target that is always out of date.
cmake_force:
.PHONY : cmake_force

#=============================================================================
# Set environment variables for the build.

# The shell in which to execute make rules.
SHELL = /bin/sh

# The CMake executable.
CMAKE_COMMAND = /usr/bin/cmake

# The command to remove a file.
RM = /usr/bin/cmake -E rm -f

# Escaping for special characters.
EQUALS = =

# The top-level source directory on which CMake was run.
CMAKE_SOURCE_DIR = /home/felix/Documents/deadlocked

# The top-level build directory on which CMake was run.
CMAKE_BINARY_DIR = /home/felix/Documents/deadlocked/build

# Include any dependencies generated for this target.
include CMakeFiles/deadlocked.dir/depend.make
# Include any dependencies generated by the compiler for this target.
include CMakeFiles/deadlocked.dir/compiler_depend.make

# Include the progress variables for this target.
include CMakeFiles/deadlocked.dir/progress.make

# Include the compile flags for this target's objects.
include CMakeFiles/deadlocked.dir/flags.make

CMakeFiles/deadlocked.dir/src/cs2/cs2.cpp.o: CMakeFiles/deadlocked.dir/flags.make
CMakeFiles/deadlocked.dir/src/cs2/cs2.cpp.o: /home/felix/Documents/deadlocked/src/cs2/cs2.cpp
CMakeFiles/deadlocked.dir/src/cs2/cs2.cpp.o: CMakeFiles/deadlocked.dir/compiler_depend.ts
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green --progress-dir=/home/felix/Documents/deadlocked/build/CMakeFiles --progress-num=$(CMAKE_PROGRESS_1) "Building CXX object CMakeFiles/deadlocked.dir/src/cs2/cs2.cpp.o"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -MD -MT CMakeFiles/deadlocked.dir/src/cs2/cs2.cpp.o -MF CMakeFiles/deadlocked.dir/src/cs2/cs2.cpp.o.d -o CMakeFiles/deadlocked.dir/src/cs2/cs2.cpp.o -c /home/felix/Documents/deadlocked/src/cs2/cs2.cpp

CMakeFiles/deadlocked.dir/src/cs2/cs2.cpp.i: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green "Preprocessing CXX source to CMakeFiles/deadlocked.dir/src/cs2/cs2.cpp.i"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -E /home/felix/Documents/deadlocked/src/cs2/cs2.cpp > CMakeFiles/deadlocked.dir/src/cs2/cs2.cpp.i

CMakeFiles/deadlocked.dir/src/cs2/cs2.cpp.s: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green "Compiling CXX source to assembly CMakeFiles/deadlocked.dir/src/cs2/cs2.cpp.s"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -S /home/felix/Documents/deadlocked/src/cs2/cs2.cpp -o CMakeFiles/deadlocked.dir/src/cs2/cs2.cpp.s

CMakeFiles/deadlocked.dir/src/globals.cpp.o: CMakeFiles/deadlocked.dir/flags.make
CMakeFiles/deadlocked.dir/src/globals.cpp.o: /home/felix/Documents/deadlocked/src/globals.cpp
CMakeFiles/deadlocked.dir/src/globals.cpp.o: CMakeFiles/deadlocked.dir/compiler_depend.ts
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green --progress-dir=/home/felix/Documents/deadlocked/build/CMakeFiles --progress-num=$(CMAKE_PROGRESS_2) "Building CXX object CMakeFiles/deadlocked.dir/src/globals.cpp.o"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -MD -MT CMakeFiles/deadlocked.dir/src/globals.cpp.o -MF CMakeFiles/deadlocked.dir/src/globals.cpp.o.d -o CMakeFiles/deadlocked.dir/src/globals.cpp.o -c /home/felix/Documents/deadlocked/src/globals.cpp

CMakeFiles/deadlocked.dir/src/globals.cpp.i: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green "Preprocessing CXX source to CMakeFiles/deadlocked.dir/src/globals.cpp.i"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -E /home/felix/Documents/deadlocked/src/globals.cpp > CMakeFiles/deadlocked.dir/src/globals.cpp.i

CMakeFiles/deadlocked.dir/src/globals.cpp.s: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green "Compiling CXX source to assembly CMakeFiles/deadlocked.dir/src/globals.cpp.s"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -S /home/felix/Documents/deadlocked/src/globals.cpp -o CMakeFiles/deadlocked.dir/src/globals.cpp.s

CMakeFiles/deadlocked.dir/src/gui.cpp.o: CMakeFiles/deadlocked.dir/flags.make
CMakeFiles/deadlocked.dir/src/gui.cpp.o: /home/felix/Documents/deadlocked/src/gui.cpp
CMakeFiles/deadlocked.dir/src/gui.cpp.o: CMakeFiles/deadlocked.dir/compiler_depend.ts
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green --progress-dir=/home/felix/Documents/deadlocked/build/CMakeFiles --progress-num=$(CMAKE_PROGRESS_3) "Building CXX object CMakeFiles/deadlocked.dir/src/gui.cpp.o"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -MD -MT CMakeFiles/deadlocked.dir/src/gui.cpp.o -MF CMakeFiles/deadlocked.dir/src/gui.cpp.o.d -o CMakeFiles/deadlocked.dir/src/gui.cpp.o -c /home/felix/Documents/deadlocked/src/gui.cpp

CMakeFiles/deadlocked.dir/src/gui.cpp.i: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green "Preprocessing CXX source to CMakeFiles/deadlocked.dir/src/gui.cpp.i"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -E /home/felix/Documents/deadlocked/src/gui.cpp > CMakeFiles/deadlocked.dir/src/gui.cpp.i

CMakeFiles/deadlocked.dir/src/gui.cpp.s: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green "Compiling CXX source to assembly CMakeFiles/deadlocked.dir/src/gui.cpp.s"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -S /home/felix/Documents/deadlocked/src/gui.cpp -o CMakeFiles/deadlocked.dir/src/gui.cpp.s

CMakeFiles/deadlocked.dir/src/main.cpp.o: CMakeFiles/deadlocked.dir/flags.make
CMakeFiles/deadlocked.dir/src/main.cpp.o: /home/felix/Documents/deadlocked/src/main.cpp
CMakeFiles/deadlocked.dir/src/main.cpp.o: CMakeFiles/deadlocked.dir/compiler_depend.ts
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green --progress-dir=/home/felix/Documents/deadlocked/build/CMakeFiles --progress-num=$(CMAKE_PROGRESS_4) "Building CXX object CMakeFiles/deadlocked.dir/src/main.cpp.o"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -MD -MT CMakeFiles/deadlocked.dir/src/main.cpp.o -MF CMakeFiles/deadlocked.dir/src/main.cpp.o.d -o CMakeFiles/deadlocked.dir/src/main.cpp.o -c /home/felix/Documents/deadlocked/src/main.cpp

CMakeFiles/deadlocked.dir/src/main.cpp.i: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green "Preprocessing CXX source to CMakeFiles/deadlocked.dir/src/main.cpp.i"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -E /home/felix/Documents/deadlocked/src/main.cpp > CMakeFiles/deadlocked.dir/src/main.cpp.i

CMakeFiles/deadlocked.dir/src/main.cpp.s: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green "Compiling CXX source to assembly CMakeFiles/deadlocked.dir/src/main.cpp.s"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -S /home/felix/Documents/deadlocked/src/main.cpp -o CMakeFiles/deadlocked.dir/src/main.cpp.s

CMakeFiles/deadlocked.dir/src/math.cpp.o: CMakeFiles/deadlocked.dir/flags.make
CMakeFiles/deadlocked.dir/src/math.cpp.o: /home/felix/Documents/deadlocked/src/math.cpp
CMakeFiles/deadlocked.dir/src/math.cpp.o: CMakeFiles/deadlocked.dir/compiler_depend.ts
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green --progress-dir=/home/felix/Documents/deadlocked/build/CMakeFiles --progress-num=$(CMAKE_PROGRESS_5) "Building CXX object CMakeFiles/deadlocked.dir/src/math.cpp.o"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -MD -MT CMakeFiles/deadlocked.dir/src/math.cpp.o -MF CMakeFiles/deadlocked.dir/src/math.cpp.o.d -o CMakeFiles/deadlocked.dir/src/math.cpp.o -c /home/felix/Documents/deadlocked/src/math.cpp

CMakeFiles/deadlocked.dir/src/math.cpp.i: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green "Preprocessing CXX source to CMakeFiles/deadlocked.dir/src/math.cpp.i"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -E /home/felix/Documents/deadlocked/src/math.cpp > CMakeFiles/deadlocked.dir/src/math.cpp.i

CMakeFiles/deadlocked.dir/src/math.cpp.s: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green "Compiling CXX source to assembly CMakeFiles/deadlocked.dir/src/math.cpp.s"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -S /home/felix/Documents/deadlocked/src/math.cpp -o CMakeFiles/deadlocked.dir/src/math.cpp.s

CMakeFiles/deadlocked.dir/src/process.cpp.o: CMakeFiles/deadlocked.dir/flags.make
CMakeFiles/deadlocked.dir/src/process.cpp.o: /home/felix/Documents/deadlocked/src/process.cpp
CMakeFiles/deadlocked.dir/src/process.cpp.o: CMakeFiles/deadlocked.dir/compiler_depend.ts
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green --progress-dir=/home/felix/Documents/deadlocked/build/CMakeFiles --progress-num=$(CMAKE_PROGRESS_6) "Building CXX object CMakeFiles/deadlocked.dir/src/process.cpp.o"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -MD -MT CMakeFiles/deadlocked.dir/src/process.cpp.o -MF CMakeFiles/deadlocked.dir/src/process.cpp.o.d -o CMakeFiles/deadlocked.dir/src/process.cpp.o -c /home/felix/Documents/deadlocked/src/process.cpp

CMakeFiles/deadlocked.dir/src/process.cpp.i: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green "Preprocessing CXX source to CMakeFiles/deadlocked.dir/src/process.cpp.i"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -E /home/felix/Documents/deadlocked/src/process.cpp > CMakeFiles/deadlocked.dir/src/process.cpp.i

CMakeFiles/deadlocked.dir/src/process.cpp.s: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green "Compiling CXX source to assembly CMakeFiles/deadlocked.dir/src/process.cpp.s"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -S /home/felix/Documents/deadlocked/src/process.cpp -o CMakeFiles/deadlocked.dir/src/process.cpp.s

CMakeFiles/deadlocked.dir/imgui/imgui.cpp.o: CMakeFiles/deadlocked.dir/flags.make
CMakeFiles/deadlocked.dir/imgui/imgui.cpp.o: /home/felix/Documents/deadlocked/imgui/imgui.cpp
CMakeFiles/deadlocked.dir/imgui/imgui.cpp.o: CMakeFiles/deadlocked.dir/compiler_depend.ts
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green --progress-dir=/home/felix/Documents/deadlocked/build/CMakeFiles --progress-num=$(CMAKE_PROGRESS_7) "Building CXX object CMakeFiles/deadlocked.dir/imgui/imgui.cpp.o"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -MD -MT CMakeFiles/deadlocked.dir/imgui/imgui.cpp.o -MF CMakeFiles/deadlocked.dir/imgui/imgui.cpp.o.d -o CMakeFiles/deadlocked.dir/imgui/imgui.cpp.o -c /home/felix/Documents/deadlocked/imgui/imgui.cpp

CMakeFiles/deadlocked.dir/imgui/imgui.cpp.i: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green "Preprocessing CXX source to CMakeFiles/deadlocked.dir/imgui/imgui.cpp.i"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -E /home/felix/Documents/deadlocked/imgui/imgui.cpp > CMakeFiles/deadlocked.dir/imgui/imgui.cpp.i

CMakeFiles/deadlocked.dir/imgui/imgui.cpp.s: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green "Compiling CXX source to assembly CMakeFiles/deadlocked.dir/imgui/imgui.cpp.s"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -S /home/felix/Documents/deadlocked/imgui/imgui.cpp -o CMakeFiles/deadlocked.dir/imgui/imgui.cpp.s

CMakeFiles/deadlocked.dir/imgui/imgui_demo.cpp.o: CMakeFiles/deadlocked.dir/flags.make
CMakeFiles/deadlocked.dir/imgui/imgui_demo.cpp.o: /home/felix/Documents/deadlocked/imgui/imgui_demo.cpp
CMakeFiles/deadlocked.dir/imgui/imgui_demo.cpp.o: CMakeFiles/deadlocked.dir/compiler_depend.ts
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green --progress-dir=/home/felix/Documents/deadlocked/build/CMakeFiles --progress-num=$(CMAKE_PROGRESS_8) "Building CXX object CMakeFiles/deadlocked.dir/imgui/imgui_demo.cpp.o"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -MD -MT CMakeFiles/deadlocked.dir/imgui/imgui_demo.cpp.o -MF CMakeFiles/deadlocked.dir/imgui/imgui_demo.cpp.o.d -o CMakeFiles/deadlocked.dir/imgui/imgui_demo.cpp.o -c /home/felix/Documents/deadlocked/imgui/imgui_demo.cpp

CMakeFiles/deadlocked.dir/imgui/imgui_demo.cpp.i: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green "Preprocessing CXX source to CMakeFiles/deadlocked.dir/imgui/imgui_demo.cpp.i"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -E /home/felix/Documents/deadlocked/imgui/imgui_demo.cpp > CMakeFiles/deadlocked.dir/imgui/imgui_demo.cpp.i

CMakeFiles/deadlocked.dir/imgui/imgui_demo.cpp.s: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green "Compiling CXX source to assembly CMakeFiles/deadlocked.dir/imgui/imgui_demo.cpp.s"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -S /home/felix/Documents/deadlocked/imgui/imgui_demo.cpp -o CMakeFiles/deadlocked.dir/imgui/imgui_demo.cpp.s

CMakeFiles/deadlocked.dir/imgui/imgui_draw.cpp.o: CMakeFiles/deadlocked.dir/flags.make
CMakeFiles/deadlocked.dir/imgui/imgui_draw.cpp.o: /home/felix/Documents/deadlocked/imgui/imgui_draw.cpp
CMakeFiles/deadlocked.dir/imgui/imgui_draw.cpp.o: CMakeFiles/deadlocked.dir/compiler_depend.ts
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green --progress-dir=/home/felix/Documents/deadlocked/build/CMakeFiles --progress-num=$(CMAKE_PROGRESS_9) "Building CXX object CMakeFiles/deadlocked.dir/imgui/imgui_draw.cpp.o"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -MD -MT CMakeFiles/deadlocked.dir/imgui/imgui_draw.cpp.o -MF CMakeFiles/deadlocked.dir/imgui/imgui_draw.cpp.o.d -o CMakeFiles/deadlocked.dir/imgui/imgui_draw.cpp.o -c /home/felix/Documents/deadlocked/imgui/imgui_draw.cpp

CMakeFiles/deadlocked.dir/imgui/imgui_draw.cpp.i: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green "Preprocessing CXX source to CMakeFiles/deadlocked.dir/imgui/imgui_draw.cpp.i"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -E /home/felix/Documents/deadlocked/imgui/imgui_draw.cpp > CMakeFiles/deadlocked.dir/imgui/imgui_draw.cpp.i

CMakeFiles/deadlocked.dir/imgui/imgui_draw.cpp.s: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green "Compiling CXX source to assembly CMakeFiles/deadlocked.dir/imgui/imgui_draw.cpp.s"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -S /home/felix/Documents/deadlocked/imgui/imgui_draw.cpp -o CMakeFiles/deadlocked.dir/imgui/imgui_draw.cpp.s

CMakeFiles/deadlocked.dir/imgui/imgui_tables.cpp.o: CMakeFiles/deadlocked.dir/flags.make
CMakeFiles/deadlocked.dir/imgui/imgui_tables.cpp.o: /home/felix/Documents/deadlocked/imgui/imgui_tables.cpp
CMakeFiles/deadlocked.dir/imgui/imgui_tables.cpp.o: CMakeFiles/deadlocked.dir/compiler_depend.ts
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green --progress-dir=/home/felix/Documents/deadlocked/build/CMakeFiles --progress-num=$(CMAKE_PROGRESS_10) "Building CXX object CMakeFiles/deadlocked.dir/imgui/imgui_tables.cpp.o"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -MD -MT CMakeFiles/deadlocked.dir/imgui/imgui_tables.cpp.o -MF CMakeFiles/deadlocked.dir/imgui/imgui_tables.cpp.o.d -o CMakeFiles/deadlocked.dir/imgui/imgui_tables.cpp.o -c /home/felix/Documents/deadlocked/imgui/imgui_tables.cpp

CMakeFiles/deadlocked.dir/imgui/imgui_tables.cpp.i: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green "Preprocessing CXX source to CMakeFiles/deadlocked.dir/imgui/imgui_tables.cpp.i"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -E /home/felix/Documents/deadlocked/imgui/imgui_tables.cpp > CMakeFiles/deadlocked.dir/imgui/imgui_tables.cpp.i

CMakeFiles/deadlocked.dir/imgui/imgui_tables.cpp.s: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green "Compiling CXX source to assembly CMakeFiles/deadlocked.dir/imgui/imgui_tables.cpp.s"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -S /home/felix/Documents/deadlocked/imgui/imgui_tables.cpp -o CMakeFiles/deadlocked.dir/imgui/imgui_tables.cpp.s

CMakeFiles/deadlocked.dir/imgui/imgui_widgets.cpp.o: CMakeFiles/deadlocked.dir/flags.make
CMakeFiles/deadlocked.dir/imgui/imgui_widgets.cpp.o: /home/felix/Documents/deadlocked/imgui/imgui_widgets.cpp
CMakeFiles/deadlocked.dir/imgui/imgui_widgets.cpp.o: CMakeFiles/deadlocked.dir/compiler_depend.ts
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green --progress-dir=/home/felix/Documents/deadlocked/build/CMakeFiles --progress-num=$(CMAKE_PROGRESS_11) "Building CXX object CMakeFiles/deadlocked.dir/imgui/imgui_widgets.cpp.o"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -MD -MT CMakeFiles/deadlocked.dir/imgui/imgui_widgets.cpp.o -MF CMakeFiles/deadlocked.dir/imgui/imgui_widgets.cpp.o.d -o CMakeFiles/deadlocked.dir/imgui/imgui_widgets.cpp.o -c /home/felix/Documents/deadlocked/imgui/imgui_widgets.cpp

CMakeFiles/deadlocked.dir/imgui/imgui_widgets.cpp.i: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green "Preprocessing CXX source to CMakeFiles/deadlocked.dir/imgui/imgui_widgets.cpp.i"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -E /home/felix/Documents/deadlocked/imgui/imgui_widgets.cpp > CMakeFiles/deadlocked.dir/imgui/imgui_widgets.cpp.i

CMakeFiles/deadlocked.dir/imgui/imgui_widgets.cpp.s: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green "Compiling CXX source to assembly CMakeFiles/deadlocked.dir/imgui/imgui_widgets.cpp.s"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -S /home/felix/Documents/deadlocked/imgui/imgui_widgets.cpp -o CMakeFiles/deadlocked.dir/imgui/imgui_widgets.cpp.s

CMakeFiles/deadlocked.dir/imgui/backends/imgui_impl_glfw.cpp.o: CMakeFiles/deadlocked.dir/flags.make
CMakeFiles/deadlocked.dir/imgui/backends/imgui_impl_glfw.cpp.o: /home/felix/Documents/deadlocked/imgui/backends/imgui_impl_glfw.cpp
CMakeFiles/deadlocked.dir/imgui/backends/imgui_impl_glfw.cpp.o: CMakeFiles/deadlocked.dir/compiler_depend.ts
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green --progress-dir=/home/felix/Documents/deadlocked/build/CMakeFiles --progress-num=$(CMAKE_PROGRESS_12) "Building CXX object CMakeFiles/deadlocked.dir/imgui/backends/imgui_impl_glfw.cpp.o"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -MD -MT CMakeFiles/deadlocked.dir/imgui/backends/imgui_impl_glfw.cpp.o -MF CMakeFiles/deadlocked.dir/imgui/backends/imgui_impl_glfw.cpp.o.d -o CMakeFiles/deadlocked.dir/imgui/backends/imgui_impl_glfw.cpp.o -c /home/felix/Documents/deadlocked/imgui/backends/imgui_impl_glfw.cpp

CMakeFiles/deadlocked.dir/imgui/backends/imgui_impl_glfw.cpp.i: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green "Preprocessing CXX source to CMakeFiles/deadlocked.dir/imgui/backends/imgui_impl_glfw.cpp.i"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -E /home/felix/Documents/deadlocked/imgui/backends/imgui_impl_glfw.cpp > CMakeFiles/deadlocked.dir/imgui/backends/imgui_impl_glfw.cpp.i

CMakeFiles/deadlocked.dir/imgui/backends/imgui_impl_glfw.cpp.s: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green "Compiling CXX source to assembly CMakeFiles/deadlocked.dir/imgui/backends/imgui_impl_glfw.cpp.s"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -S /home/felix/Documents/deadlocked/imgui/backends/imgui_impl_glfw.cpp -o CMakeFiles/deadlocked.dir/imgui/backends/imgui_impl_glfw.cpp.s

CMakeFiles/deadlocked.dir/imgui/backends/imgui_impl_opengl3.cpp.o: CMakeFiles/deadlocked.dir/flags.make
CMakeFiles/deadlocked.dir/imgui/backends/imgui_impl_opengl3.cpp.o: /home/felix/Documents/deadlocked/imgui/backends/imgui_impl_opengl3.cpp
CMakeFiles/deadlocked.dir/imgui/backends/imgui_impl_opengl3.cpp.o: CMakeFiles/deadlocked.dir/compiler_depend.ts
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green --progress-dir=/home/felix/Documents/deadlocked/build/CMakeFiles --progress-num=$(CMAKE_PROGRESS_13) "Building CXX object CMakeFiles/deadlocked.dir/imgui/backends/imgui_impl_opengl3.cpp.o"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -MD -MT CMakeFiles/deadlocked.dir/imgui/backends/imgui_impl_opengl3.cpp.o -MF CMakeFiles/deadlocked.dir/imgui/backends/imgui_impl_opengl3.cpp.o.d -o CMakeFiles/deadlocked.dir/imgui/backends/imgui_impl_opengl3.cpp.o -c /home/felix/Documents/deadlocked/imgui/backends/imgui_impl_opengl3.cpp

CMakeFiles/deadlocked.dir/imgui/backends/imgui_impl_opengl3.cpp.i: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green "Preprocessing CXX source to CMakeFiles/deadlocked.dir/imgui/backends/imgui_impl_opengl3.cpp.i"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -E /home/felix/Documents/deadlocked/imgui/backends/imgui_impl_opengl3.cpp > CMakeFiles/deadlocked.dir/imgui/backends/imgui_impl_opengl3.cpp.i

CMakeFiles/deadlocked.dir/imgui/backends/imgui_impl_opengl3.cpp.s: cmake_force
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green "Compiling CXX source to assembly CMakeFiles/deadlocked.dir/imgui/backends/imgui_impl_opengl3.cpp.s"
	/usr/bin/g++ $(CXX_DEFINES) $(CXX_INCLUDES) $(CXX_FLAGS) -S /home/felix/Documents/deadlocked/imgui/backends/imgui_impl_opengl3.cpp -o CMakeFiles/deadlocked.dir/imgui/backends/imgui_impl_opengl3.cpp.s

# Object files for target deadlocked
deadlocked_OBJECTS = \
"CMakeFiles/deadlocked.dir/src/cs2/cs2.cpp.o" \
"CMakeFiles/deadlocked.dir/src/globals.cpp.o" \
"CMakeFiles/deadlocked.dir/src/gui.cpp.o" \
"CMakeFiles/deadlocked.dir/src/main.cpp.o" \
"CMakeFiles/deadlocked.dir/src/math.cpp.o" \
"CMakeFiles/deadlocked.dir/src/process.cpp.o" \
"CMakeFiles/deadlocked.dir/imgui/imgui.cpp.o" \
"CMakeFiles/deadlocked.dir/imgui/imgui_demo.cpp.o" \
"CMakeFiles/deadlocked.dir/imgui/imgui_draw.cpp.o" \
"CMakeFiles/deadlocked.dir/imgui/imgui_tables.cpp.o" \
"CMakeFiles/deadlocked.dir/imgui/imgui_widgets.cpp.o" \
"CMakeFiles/deadlocked.dir/imgui/backends/imgui_impl_glfw.cpp.o" \
"CMakeFiles/deadlocked.dir/imgui/backends/imgui_impl_opengl3.cpp.o"

# External object files for target deadlocked
deadlocked_EXTERNAL_OBJECTS =

deadlocked: CMakeFiles/deadlocked.dir/src/cs2/cs2.cpp.o
deadlocked: CMakeFiles/deadlocked.dir/src/globals.cpp.o
deadlocked: CMakeFiles/deadlocked.dir/src/gui.cpp.o
deadlocked: CMakeFiles/deadlocked.dir/src/main.cpp.o
deadlocked: CMakeFiles/deadlocked.dir/src/math.cpp.o
deadlocked: CMakeFiles/deadlocked.dir/src/process.cpp.o
deadlocked: CMakeFiles/deadlocked.dir/imgui/imgui.cpp.o
deadlocked: CMakeFiles/deadlocked.dir/imgui/imgui_demo.cpp.o
deadlocked: CMakeFiles/deadlocked.dir/imgui/imgui_draw.cpp.o
deadlocked: CMakeFiles/deadlocked.dir/imgui/imgui_tables.cpp.o
deadlocked: CMakeFiles/deadlocked.dir/imgui/imgui_widgets.cpp.o
deadlocked: CMakeFiles/deadlocked.dir/imgui/backends/imgui_impl_glfw.cpp.o
deadlocked: CMakeFiles/deadlocked.dir/imgui/backends/imgui_impl_opengl3.cpp.o
deadlocked: CMakeFiles/deadlocked.dir/build.make
deadlocked: glfw/src/libglfw3.a
deadlocked: glm/glm/libglm.a
deadlocked: /usr/lib64/librt.a
deadlocked: /usr/lib64/libm.so
deadlocked: CMakeFiles/deadlocked.dir/link.txt
	@$(CMAKE_COMMAND) -E cmake_echo_color "--switch=$(COLOR)" --green --bold --progress-dir=/home/felix/Documents/deadlocked/build/CMakeFiles --progress-num=$(CMAKE_PROGRESS_14) "Linking CXX executable deadlocked"
	$(CMAKE_COMMAND) -E cmake_link_script CMakeFiles/deadlocked.dir/link.txt --verbose=$(VERBOSE)

# Rule to build all files generated by this target.
CMakeFiles/deadlocked.dir/build: deadlocked
.PHONY : CMakeFiles/deadlocked.dir/build

CMakeFiles/deadlocked.dir/clean:
	$(CMAKE_COMMAND) -P CMakeFiles/deadlocked.dir/cmake_clean.cmake
.PHONY : CMakeFiles/deadlocked.dir/clean

CMakeFiles/deadlocked.dir/depend:
	cd /home/felix/Documents/deadlocked/build && $(CMAKE_COMMAND) -E cmake_depends "Unix Makefiles" /home/felix/Documents/deadlocked /home/felix/Documents/deadlocked /home/felix/Documents/deadlocked/build /home/felix/Documents/deadlocked/build /home/felix/Documents/deadlocked/build/CMakeFiles/deadlocked.dir/DependInfo.cmake "--color=$(COLOR)"
.PHONY : CMakeFiles/deadlocked.dir/depend

