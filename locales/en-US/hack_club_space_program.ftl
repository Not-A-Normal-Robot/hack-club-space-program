general__back = back

mainMenu__playButton__text = play
mainMenu__aboutButton__text = about
mainMenu__quitButton__text = quit

gameControlMode__tooltip =
    This is your current game control mode.
    This determines what actions you can do right now with your keyboard.
    This is similar to Vim's modal architecture, where you can switch between
    modes easily.
gameControlMode__mainMode = main
gameControlMode__menuMode = menu
gameControlMode__vesselControlMode = vessel
gameControlMode__cameraControlMode = camera

aboutMenu__backButton__text = { general__back }
aboutMenu__title__text = About
aboutMenu__article__main__title = About Hack Club Space Program
aboutMenu__article__main__body =
    Hack Club Space Program is an open-source game originally made for Hack Club Flavortown.
    This game is heavily inspired by similar games such as Kerbal Space Program and Spaceflight Simulator.

    The license text of this project as well as used assets are contained within this About page.
    
    This game was originally made by Not-A-Normal-Robot: <https://github.com/Not-A-Normal-Robot/>
    The source code of the game is available online in <https://github.com/Not-A-Normal-Robot/hack-club-space-program/>.
    
    Hack Club Flavortown is a hackathon for teenagers hosted by Hack Club: <https://flavortown.hackclub.com/>

    # Gameplay

    This WIP project has next to no gameplay. You aren't able to interact with anything yet, but you can view the rudimentary gravity/orbit simulation. Most of the work still currently resides in the backend.

    In the bottom, there is an oribar (orientation bar) indicating the current orientation/angle that the vessel (the spinning rectangle) is currently facing. You'll also see a velocity indicator on the bottom right and the current control mode in the bottom left.

    In the top right, you will see the altitude indicator. It has three modes you can switch between by clicking on it:
    1. ASL: Above Sea Level. Displays the altitude above the sea level, or in the case of sealess celestial bodies, an arbitrary offset from the center of the celestial body.
    2. AGL: Above Ground Level. Displays the altitude above terrain.
    3. CTR: Center. Displays the distance between the vessel and the center of the celestial body.

    ## Control
    The control scheme is very incomplete, and right now you can only really move the camera around.
    Like Vim, there are a couple control modes you can choose between.

    ### "Main" Control Mode
    This is the control mode you get put in when you start the game.
    This serves as a "hub" for different control modes.
    You can always press Escape to return to this mode.

    In this mode, you can:
    - Press `C` to enter "camera control" mode.
    - ...All other modes are currently unimplemented/stubs.

    ### "Camera control" mode
    This is the mode where you can control the camera.

    Here, **Shift** and **Ctrl** are speed modifiers which you can hold down to change the speed of camera movements. **Shift** makes camera movements faster, while **Ctrl** makes them slower.

    In this mode, you can:
    - Hold WASD or arrow keys to pan the camera relative to its focus
    - Press C to re-Center the camera on its focus
    - Hold Q or E to rotate the camera
    - Press R to reset the Rotation of the camera
    - Press - or + to alter the zoom of the camera
    - Press 0 to reset the zoom of the camera
    - Press [ or ] to switch the focus of the camera between multiple objects. (This currently does nothing)

aboutMenu__article__gameLicense__title = License: Hack Club Space Program
aboutMenu__article__dotoLicense__title = Font license: Doto
aboutMenu__article__wdxlLicense__title = Font license: WDXL Lubrifont SC
aboutMenu__article__jbmLicense__title = Font license: JetBrains Mono

altimeter__altitude__label = ALTITUDE
altimeter__mode__asl__text = ASL
altimeter__mode__asl__tooltip = Above Sea Level
altimeter__mode__agl__text = AGL
altimeter__mode__agl__tooltip = Above Ground Level
altimeter__mode__ctr__text = CTR
altimeter__mode__ctr__tooltip = From Planetary Centre

speedometer__horizontalSpeed__label = HSPD
speedometer__horizontalSpeed__tooltip = Horizontal speed relative to parent body
speedometer__verticalSpeed__label = VSPD
speedometer__verticalSpeed__tooltip = Vertical speed relative to parent body
speedometer__totalSpeed__label = SPD
speedometer__totalSpeed__tooltip = Speed relative to parent body
