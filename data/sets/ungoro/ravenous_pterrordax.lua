local options = {
    "UNG_999t2", "UNG_999t3", "UNG_999t4", "UNG_999t5", "UNG_999t6",
    "UNG_999t7", "UNG_999t8", "UNG_999t10", "UNG_999t13", "UNG_999t14",
}

local function apply(ctx, target, choice)
    if ctx:entity(target).zone ~= "board" then return end
    if choice == "UNG_999t2" then ctx:attach_deathrattle(target, "UNG_999t2"); ctx:grant_keyword(target, "deathrattle")
    elseif choice == "UNG_999t3" then ctx:buff(target, 3, 0)
    elseif choice == "UNG_999t4" then ctx:buff(target, 0, 3)
    elseif choice == "UNG_999t5" then ctx:grant_keyword(target, "elusive")
    elseif choice == "UNG_999t6" then ctx:grant_keyword(target, "taunt")
    elseif choice == "UNG_999t7" then ctx:grant_keyword(target, "windfury")
    elseif choice == "UNG_999t8" then ctx:grant_keyword(target, "divine_shield")
    elseif choice == "UNG_999t10" then ctx:grant_keyword_until_next_turn(target, "stealth")
    elseif choice == "UNG_999t13" then ctx:grant_keyword(target, "poisonous")
    elseif choice == "UNG_999t14" then ctx:buff(target, 1, 1) end
end

local card = {
    api_version = 1, id = "UNG_047", name = "Ravenous Pterrordax",
    text = "<b>Battlecry:</b> Destroy a friendly minion to <b>Adapt</b> twice.",
    set = "UNGORO", type = "minion", class = "warlock", rarity = "common",
    cost = 3, attack = 3, health = 3, tags = { "beast" }, keywords = { "battlecry" },
    target_mode = "required_if_available",
}
function card.targets(ctx, self)
    local result = {}
    for _, entity in ipairs(ctx:friendly_minions(self)) do if entity ~= self then result[#result + 1] = entity end end
    return result
end
function card.on_battlecry(ctx, self, target)
    if not target then return end
    ctx:destroy(target)
    ctx:set_data(self, "adapts_left", 2)
    ctx:continue_with("choose_adapt")
end
function card.choose_adapt(ctx, self)
    if ctx:entity(self).zone == "board" and ctx:get_data(self, "adapts_left") > 0 then
        ctx:discover_cards(ctx:controller(self), "Adapt", options, 3, "adapted")
    end
end
function card.adapted(ctx, self, choice)
    apply(ctx, self, choice)
    local left = ctx:get_data(self, "adapts_left") - 1
    ctx:set_data(self, "adapts_left", left)
    if left > 0 then ctx:continue_with("choose_adapt") end
end
return card
