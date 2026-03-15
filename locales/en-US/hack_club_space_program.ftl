general__back = back

error__saveGeneral__noSaveDir =
    We couldn't determine a reasonable location for save files.
error__saveGeneral__ioError =
    There was an I/O error trying to read the save file:
    { $inner }

error__saveData__rootCelestialNotFound = Couldn't find root celestial body.
error__saveData__activeVesselNotFound = Couldn't find active vessel.
error__saveData__celestialNotFound =
    Celestial body with ID { $referrer } had a reference to a nonexistent celestial body with ID { $not_found }.
error__saveData__vesselNotFound =
    Celestial body with ID { $referrer } had a reference to a nonexistent vessel with ID { $not_found }.
error__saveData__duplicateCelestial =
    Celestial body with ID { $duplicated } appears more than once in references: { $first_referrer ->
        [none] it's referenced as the root element and the child of the celestial body with ID { $second_referrer }.
        *[some] it's referenced as a child of celestial bodies with IDs { $first_referrer } and { $second_referrer }.
    }
error__saveData__duplicateVessel =
    Vessel with ID { $duplicated } appears more than once in references: it's referenced as a child of celestial bodies with IDs { $first_referrer } and { $second_referrer }.
error__saveData__orphanedCelestials =
    The save file contains celestial bodies without a parent: { $list }
error__saveData__orphanedVessels =
    The save file contains vessels without a parent: { $list }

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