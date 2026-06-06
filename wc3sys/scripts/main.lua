local function fourcc(s)
    return string.byte(s, 1) * 0x1000000
        + string.byte(s, 2) * 0x10000
        + string.byte(s, 3) * 0x100
        + string.byte(s, 4)
end

local function register_frame_click(frame, action)
    local trigger = CreateTrigger()
    if not trigger or trigger == 0 then
        print("CreateTrigger failed for frame click")
        return 0
    end

    BlzTriggerRegisterFrameEvent(trigger, frame, 1)
    TriggerAddAction(trigger, action)
    return trigger
end

local function create_icon_button(parent, x, y, texture, callback)
    local button = BlzCreateFrame("ScriptDialogButton", parent, 0, 0)
    if not button or button == 0 then
        return 0, 0
    end

    local icon = BlzCreateFrame("EscMenuBackdrop", button, 0, 0)

    BlzFrameClearAllPoints(button)
    BlzFrameSetSize(button, 0.060, 0.060)
    BlzFrameSetAbsPoint(button, 4, x, y)
    BlzFrameShow(button, 1)

    if icon and icon ~= 0 then
        BlzFrameClearAllPoints(icon)
        BlzFrameSetSize(icon, 0.065, 0.065)
        BlzFrameSetAbsPoint(icon, 4, x, y)
        BlzFrameSetTexture(icon, texture, 0, 1)
        BlzFrameShow(icon, 1)
    end

    register_frame_click(button, callback)
    return button, icon
end

local function first_selected_unit(player)
    local group = CreateGroup()
    if not group or group == 0 then
        return nil
    end

    GroupEnumUnitsSelected(group, player, 0)
    local unit = FirstOfGroup(group)
    DestroyGroup(group)

    if unit and unit ~= 0 then
        return unit
    end

    return nil
end

function main()
    local player = Player(0)
    local gameUI = BlzGetGameUI()
    if not gameUI or gameUI == 0 then
        print("BlzGetGameUI failed")
        return
    end

    BlzLoadTOCFile("UI\\FrameDef\\FrameDef.toc")

    local consoleOpen = false
    local visualOpen = false
    local consoleFrames = {}
    local visualFrames = {}
    local spawnedUnits = {}
    local lastSpawnedUnit = nil

    local function show_console(show)
        consoleOpen = show

        for _, frame in ipairs(consoleFrames) do
            if frame and frame ~= 0 then
                BlzFrameShow(frame, show and 1 or 0)
            end
        end
    end

    local function show_visual_menu(show)
        visualOpen = show

        for _, frame in ipairs(visualFrames) do
            if frame and frame ~= 0 then
                BlzFrameShow(frame, show and 1 or 0)
            end
        end
    end

    local function get_effect_target()
        local unit = first_selected_unit(player)
        if unit and unit ~= 0 then
            return unit
        end

        if lastSpawnedUnit and lastSpawnedUnit ~= 0 then
            return lastSpawnedUnit
        end

        return nil
    end

    local function apply_visual_buff(rawcode, name, flaga,flagb)
        local unit = get_effect_target()
        if not unit or unit == 0 then
            print("No selected/spawned unit to apply " .. name)
            return
        end

        if ApplyBuff(unit, rawcode, 10.00) then
            print("Applied visual buff: " .. name)
        else
            print("Failed to apply visual buff: " .. name)
        end
    end

    local function spawn_hero(unitId, name)
        local unit = CreateUnit(player, fourcc(unitId), 0.0, 0.0, 270.0)
        if not unit or unit == 0 then
            print("Failed to spawn " .. name)
            return
        end

        SetHeroLevel(unit, 10, false)
        table.insert(spawnedUnits, unit)
        lastSpawnedUnit = unit

        print("Spawned " .. name .. " (#" .. #spawnedUnits .. ")")
    end

    -- Console/unit-spawn menu.
    local menuBackdrop = BlzCreateFrame("EscMenuBackdrop", gameUI, 0, 0)
    local title = BlzCreateFrame("EscMenuMainPanelDialogTextTemplate", gameUI, 0, 0)

    table.insert(consoleFrames, menuBackdrop)
    table.insert(consoleFrames, title)

    BlzFrameClearAllPoints(menuBackdrop)
    BlzFrameSetSize(menuBackdrop, 0.360, 0.210)
    BlzFrameSetAbsPoint(menuBackdrop, 4, 0.400, 0.330)
    BlzFrameSetTexture(menuBackdrop, "UI\\Widgets\\EscMenu\\Human\\blank-background.blp", 0, 1)

    BlzFrameClearAllPoints(title)
    BlzFrameSetSize(title, 0.260, 0.035)
    BlzFrameSetAbsPoint(title, 4, 0.400, 0.400)
    BlzTextFrameSetText(title, "Cheat Console")
    BlzFrameSetTextColor(title, 0xFFFFFFFF)

    local units = {
        {
            id = "Nplh",
            name = "Pit Lord",
            x = 0.305,
            texture = "ReplaceableTextures\\CommandButtons\\BTNPitLord.blp",
        },
        {
            id = "Ekee",
            name = "Keeper",
            x = 0.400,
            texture = "ReplaceableTextures\\CommandButtons\\BTNKeeperOfTheGrove.blp",
        },
        {
            id = "Nfir",
            name = "Firelord",
            x = 0.495,
            texture = "ReplaceableTextures\\CommandButtons\\BTNHeroAvatarOfFlame.blp",
        },
    }

    for _, spec in ipairs(units) do
        local button, icon = create_icon_button(gameUI, spec.x, 0.320, spec.texture, function()
            spawn_hero(spec.id, spec.name)
        end)

        local label = BlzCreateFrame("EscMenuMainPanelDialogTextTemplate", gameUI, 0, 0)

        table.insert(consoleFrames, button)
        table.insert(consoleFrames, icon)
        table.insert(consoleFrames, label)

        if label and label ~= 0 then
            BlzFrameClearAllPoints(label)
            BlzFrameSetSize(label, 0.090, 0.030)
            BlzFrameSetAbsPoint(label, 4, spec.x, 0.265)
            BlzTextFrameSetText(label, spec.name)
            BlzFrameSetTextColor(label, 0xFFFFFFFF)
        end
    end

    -- Visual buff picker menu.
    local visualBackdrop = BlzCreateFrame("EscMenuBackdrop", gameUI, 0, 0)
    local visualTitle = BlzCreateFrame("EscMenuMainPanelDialogTextTemplate", gameUI, 0, 0)

    table.insert(visualFrames, visualBackdrop)
    table.insert(visualFrames, visualTitle)

    BlzFrameClearAllPoints(visualBackdrop)
    BlzFrameSetSize(visualBackdrop, 0.360, 0.190)
    BlzFrameSetAbsPoint(visualBackdrop, 4, 0.400, 0.330)
    BlzFrameSetTexture(visualBackdrop, "UI\\Widgets\\EscMenu\\Human\\blank-background.blp", 0, 1)

    BlzFrameClearAllPoints(visualTitle)
    BlzFrameSetSize(visualTitle, 0.280, 0.035)
    BlzFrameSetAbsPoint(visualTitle, 4, 0.400, 0.390)
    BlzTextFrameSetText(visualTitle, "Visual Buffs")
    BlzFrameSetTextColor(visualTitle, 0xFFFFFFFF)

    local visualBuffs = {
        {
            name = "Slow",
            rawcode = fourcc("Bslo"),
            x = 0.305,
            texture = "ReplaceableTextures\\CommandButtons\\BTNSlow.blp",
			flaga=0,
			flagb=1,
        },
        {
            name = "Divine Shield",
            rawcode = fourcc("BHds"),
            x = 0.400,
            texture = "ReplaceableTextures\\CommandButtons\\BTNDivineIntervention.blp",
			flaga=1,
			flagb=1,
        },
        {
            name = "Roots",
            rawcode = fourcc("BEer"),
            x = 0.495,
            texture = "ReplaceableTextures\\CommandButtons\\BTNEntanglingRoots.blp",
			flaga=1,
			flagb=0,
        },
    }

    for _, spec in ipairs(visualBuffs) do
        local button, icon = create_icon_button(gameUI, spec.x, 0.320, spec.texture, function()
            apply_visual_buff(spec.rawcode, spec.name,spec.flaga,spec.flagb)
        end)

        local label = BlzCreateFrame("EscMenuMainPanelDialogTextTemplate", gameUI, 0, 0)

        table.insert(visualFrames, button)
        table.insert(visualFrames, icon)
        table.insert(visualFrames, label)

        if label and label ~= 0 then
            BlzFrameClearAllPoints(label)
            BlzFrameSetSize(label, 0.090, 0.030)
            BlzFrameSetAbsPoint(label, 4, spec.x, 0.265)
            BlzTextFrameSetText(label, spec.name)
            BlzFrameSetTextColor(label, 0xFFFFFFFF)
        end
    end

    show_console(false)
    show_visual_menu(false)

    -- Console toggle button.
    local toggleButton = BlzCreateFrame("ScriptDialogButton", gameUI, 0, 0)
    local toggleIcon = BlzCreateFrame("EscMenuBackdrop", toggleButton, 0, 0)

    BlzFrameClearAllPoints(toggleButton)
    BlzFrameSetSize(toggleButton, 0.045, 0.045)
    BlzFrameSetAbsPoint(toggleButton, 0, 0.030, 0.570)
    BlzFrameShow(toggleButton, 1)

    BlzFrameClearAllPoints(toggleIcon)
    BlzFrameSetSize(toggleIcon, 0.050, 0.050)
    BlzFrameSetAbsPoint(toggleIcon, 0, 0.025, 0.5725)
    BlzFrameSetTexture(toggleIcon, "ReplaceableTextures\\CommandButtons\\BTNCallToArms.blp", 0, 1)
    BlzFrameShow(toggleIcon, 1)

    register_frame_click(toggleButton, function()
        local nextState = not consoleOpen
        if nextState then
            show_visual_menu(false)
        end

        show_console(nextState)

        if consoleOpen then
            BlzFrameSetTexture(toggleIcon, "ReplaceableTextures\\CommandButtons\\BTNCancel.blp", 0, 1)
            print("Cheat console opened")
        else
            BlzFrameSetTexture(toggleIcon, "ReplaceableTextures\\CommandButtons\\BTNCallToArms.blp", 0, 1)
            print("Cheat console closed")
        end
    end)

    -- Visual buff picker toggle button.
    local visualToggleButton = BlzCreateFrame("ScriptDialogButton", gameUI, 0, 0)
    local visualToggleIcon = BlzCreateFrame("EscMenuBackdrop", visualToggleButton, 0, 0)

    BlzFrameClearAllPoints(visualToggleButton)
    BlzFrameSetSize(visualToggleButton, 0.045, 0.045)
    BlzFrameSetAbsPoint(visualToggleButton, 0, 0.085, 0.570)
    BlzFrameShow(visualToggleButton, 1)

    BlzFrameClearAllPoints(visualToggleIcon)
    BlzFrameSetSize(visualToggleIcon, 0.050, 0.050)
    BlzFrameSetAbsPoint(visualToggleIcon, 0, 0.080, 0.5725)
    BlzFrameSetTexture(visualToggleIcon, "ReplaceableTextures\\CommandButtons\\BTNSelectHeroOn.blp", 0, 1)
    BlzFrameShow(visualToggleIcon, 1)

    register_frame_click(visualToggleButton, function()
        local nextState = not visualOpen
        if nextState then
            show_console(false)
            BlzFrameSetTexture(toggleIcon, "ReplaceableTextures\\CommandButtons\\BTNCallToArms.blp", 0, 1)
        end

        show_visual_menu(nextState)

        if visualOpen then
            BlzFrameSetTexture(visualToggleIcon, "ReplaceableTextures\\CommandButtons\\BTNCancel.blp", 0, 1)
            print("Visual buff picker opened")
        else
            BlzFrameSetTexture(visualToggleIcon, "ReplaceableTextures\\CommandButtons\\BTNSelectHeroOn.blp", 0, 1)
            print("Visual buff picker closed")
        end
    end)

    print("Cheat console and visual buff picker ready")
end
