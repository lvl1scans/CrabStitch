<div align="center">
  <a href="https://github.com/lvl1scans/CrabStitch">
    <img alt="CrabStitch.Logo" width="200" height="200" src="https://raw.githubusercontent.com/lvl1scans/CrabStitch/master/logo.png">
  </a>
  <h1>CrabStitch</h1>
  <p>
    A small yet powerful program for stitching and cutting webtoons/manhwa/manhua raws.
  </p>
  <p>
    Precompiled installers available for most of the popular Operating Systems.
  </p>
  <a href="https://github.com/lvl1scans/CrabStitch/releases/latest">
    <img src="https://img.shields.io/github/v/release/lvl1scans/CrabStitch">
  </a>
  <a href="https://github.com/lvl1scans/CrabStitch/releases/latest">
    <img src="https://img.shields.io/github/release-date/lvl1scans/CrabStitch">
  </a>
  <a href="https://github.com/lvl1scans/CrabStitch/releases/">
    <img src="https://img.shields.io/github/downloads/lvl1scans/CrabStitch/total">
  </a>
  <a href="https://github.com/lvl1scans/CrabStitch/tree/dev">
    <img src="https://img.shields.io/github/last-commit/lvl1scans/CrabStitch">
  </a>
  <a href="https://github.com/lvl1scans/CrabStitch/blob/dev/LICENSE">
    <img src="https://img.shields.io/github/license/lvl1scans/CrabStitch">
  </a>
</div>

## What is CrabStitch?
A Rust based port of [SmartStitch](https://github.com/MechTechnology/SmartStitch) which should not exist but it does!

A small yet powerful program for stitching together webtoons/manhwa/manhua raws then slicing them down to the whatever size you wish for.

The smart part of the name comes from the fact that it uses some simple pixel calculation to stop itself from cutting/slicing through sfx or speech or drawings. it making life much easier for the team working on those raw images. [Both CLRD and TS will thank you a lot].

*It's not fancy, and does not use AI, but it's fast, robust, simple and more importantly works for me. (So i decided to share it with you!)*


## Screenshots
<div align="center">
<img alt="screenshot01" src="https://raw.githubusercontent.com/lvl1scans/CrabStitch/master/screenshots/1.jpeg">
<img alt="screenshot02" src="https://raw.githubusercontent.com/lvl1scans/CrabStitch/master/screenshots/2.jpeg">
<img alt="screenshot03" src="https://raw.githubusercontent.com/lvl1scans/CrabStitch/master/screenshots/3.jpeg">
</div>

## Basic Quick Get Started GUI Version
1. Open the application.
2. Browse to your raw folder.
4. Select a the output file type. (Supported types: png, jpg, webp, bmp, psd, tiff, tga)
3. Set the Rough Panel Height of the output files.
5. Click start process.
6. Done, Enjoy!

- Your file will be ordered the same way they are in your file explorer, so make sure everything is in order. (sort by name in file explorer)
- You can explore the advanced settings after reading documentation to have more control on the output files.

### How to launch the GUI Version (For All Users):
1. Put the raws you wish to stitch in a folder
2. Download the installer of program for your Operating System from latest release (Found in the releases section in this github)
3. Install the program.
4. Now launch the application, and you can proceed with the Quick get started steps.

# Documentation
Here is the complete documentation for the application, it is broken down into 4 sections, basic settings, advanced settings, how to build your own version, how to run the console version.

## Basic Settings
These are the required settings that all users should be mindful of.

### Input Folder Path
Here you have to set the path for the Input Folder which contains the raws that will be processed by the program. If batch mode is enabled, it will search for subfolder within the given input path. So make sure your folder and files are in order.

*Supported Types: png, jpg, webp, bmp, psd, avif*

### Output type
The default output type is png since it is lossless, however you can always change to other types, such as jpg, the program does save jpg at 100 quality, so there should be not noticeable loss in quality but it is up to the user what format they want. (PSD input is supported but Output is not supported because of rust crates limitation. Waiting for this PR to merge chinedufn/psd#63)

*Default: .png* --- *Supported Types: png, jpg, webp, bmp, avif*

### Rough Output Height
Here you set the size that you want most output panels to roughly be, the program will uses it as a guide to see where to slice/cut the images, however it IS ROUGH, meaning if the program finds bubbles/sfx/whatever at that specific pixel length, it will try to find the next closest position where it can cut the image. Thus the output size of each image will vary because of that, but they all will be roughly around this size.

*Default: 5000*

### Width Enforcement Mode and Custom Width
So essentially it's very straightforward. It adds a setting to select one of three modes to enforce change on the image width.
0 => No Enforcement, where you load the files as is, and work on them, if they vary in size, you will get some black lines in the side (Highest quality as there is no changes to the pixel values)
1 => Automatic uniform width, where you force all files to have the same width as the first file in the input folder.
2 => Match Minimum, where you force all files to have the same width as the minimum width file in the input folder.
3 => User Customized width, where the user specifies the width they want, that is the Custom Width parameter.
(Please just use waifu2x for upscaling raws, do not use this mode for it.)
4 => Match Maximum, where you force all files to have the same width as the maximum width file in the input folder. It adds black colums on sides of smaller files.

*Default: 1*

### Batch Mode
You can have multiple chapter folders in the input folder. The program will search the nested tree, and treat every folder within the input folder as its own chapter and will work on them.

## Advanced Settings
These are settings for more tech savvy people, or people that find themselves in a special case that need some fine tuning of the settings.

### Detector Type
Detector type is a very simple setting, currently there is a smart pixel comparison detector which is the default way of edge detection in this program, and there is Direct Slicing, which cuts all panels to the exact size that the user inputs in the rough panel height field.

*Default: Smart Pixel Comparison*

### Object Detection Senstivity (Percentage)
Before slicing at a specific height, the program checks the row of pixels it will slice at if there is bubbles/sfx/whatever, it compares neighbouring pixels for any drastic jump in value, (the allowed tolarence for jumps in pixel is the Object Detection Senstivity)

if there is too big of a jump in value between the pixels, that means there is something that shouldn't be cut, so it move up a pixel row and repeat. For 100 Senstivity will mean if entire pixel row does not have the same exact pixel value/color, it will not slice at it. For 0 Senstivity being it does not care about the pixel values and will cut there, essentially turning the program into a normal Dumb Image Slicer.

*Default: 90* --- *Value Range: 0-100*

### Scan Line Step
This is the step at which the program moves if it find the line it's on to be unsuitable to be sliced, meaning when it move on to the next line, it moves up/down X number of pixels to a new line, then it begins its scan algorithm once again. This X number of pixels is the scan line step. Smaller steps should give better results but larger ones do save computational power.

*Default: 5* --- *Value Range: 1-100*

### Ignorable Horizental Margins Pixels
This gives the option to ignore pixels on the border of the image when checking for bubbles/sfw/whatever. Why you might ask, Borders do not make the detection algorithm happy, so in some cases you want it to start its detection only inside said border, be careful to what value you want it to be since if it's larger that image it will case the program to crash/stop its operation.

*Default: 0*

#### Visualization of Ignorable Border Pixels and Scan Line Step
Red being the area ignored because of the Ignorable Border Pixels, and the blue lines would be the lines that application test for where it can slice (This example does not use the default values for those parameters)
<div align="center">
  <img alt="screenshot03" src="https://i.imgur.com/ipU6cJS.png">
</div>

### Settings Profile
For those working on various projects that require different stitching settings for each of them, you can now have multiple settings profile, that you can create and name as you like. Selecting the profile from dropdown will update all the programming settings to that of selected profile, this can for example be very useful when working with manhwas and manhuas of different resolutions.

This is setting is for convenience mainly for heavy users.

### Post Process
(GUI Only) With this option, one can set a specific console process to be fire on the output files of the application. For example, you can set it to fire waifu2x on the output files, so you can have the best raw processing experience. So how do we set that up,
  1. Navigate to the Post Process Tab
  2. Enable the run postprocess after completion flag.
  3. Set the process path/location, you can essentially browse to the process' exe file
  4. Set the arguments you want to pass to the process (Use the argument {output} to pass the output directory to your process).

#### Visualization of After Completion Subprocess (Setup for waifu2x-caffe)
Of course you can use whatever version of waifu2x or process that you want, this is just an example of what i setup for myself.
<div align="center">
  <img alt="screenshot04" src="https://i.imgur.com/fZbP1sn.png">
</div>

## How to build/compile your own GUI Version?

### How to compile GUI package (For All Users, developers only!!)
1. Install Rust, nodejs and NASM. May require additional things on Linux (check github action workflow for more details)
2. Run command `npm install` and then `npm run tauri build` in Source Code of the repo. It'll generate platform specific local installer.

## Features to port from python to rust
- [x] Automatic Pixel Detection and Stitching logic
- [x] Direct Slice mode
- [x] Width enforcement modes
- [x] Batch Mode
- [x] Post Process command
- [x] Profiles
- [x] BMP Support
- [x] AVIF Support
- [ ] PSD Support
  - [x] PSD input
  - [ ] PSD output (waiting for chinedufn/psd#63 to merge)

## More INFO about this project
This is a Tauri and Rust based port of original SmartStitch by MechTechnology. It was merely developed to test capabilities of Gemini 3 Pro. I had no intentions of maintaining yet another stitching tool.
But since it worked pretty well, I have decided to put it on github and release compiled installers for all major platforms.
This way, even people using MacOS or Linux can enjoy 'download and run' experience, instead of requiring python.

## Advantages
- Upto 70% reduction in RAM usage
- Upto 70% reduction in time taken to stitch same amount of pages
- Results identical to original SmartStitch

## Disadvantages
- AI generated code.
- No intentions of adding any more features (PRs are welcome)
- Rust code so has some learning curve for interested maintainers

### My thoughts
Since this is an AI generated application, I don't know about LICENSE and stuff to use.
I'll just advice to not use this in actual professional work.