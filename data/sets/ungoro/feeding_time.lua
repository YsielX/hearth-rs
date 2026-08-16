local options = {
    "UNG_999t2", "UNG_999t3", "UNG_999t4", "UNG_999t5", "UNG_999t6",
    "UNG_999t7", "UNG_999t8", "UNG_999t10", "UNG_999t13", "UNG_999t14",
}
local function apply(ctx, target, choice)
    if ctx:entity(target).zone ~= "board" then return end
    if choice == "UNG_999t2" then ctx:attach_hook(target, "on_deathrattle", "UNG_999t2"); ctx:grant_keyword(target, "deathrattle")
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
    api_version = 1, id = "UNG_834", name = "Feeding Time",
    text = "Deal $3 damage to a minion. Summon three 1/1 Pterrordaxes and <b>Adapt</b> them.",
    set = "UNGORO", type = "spell", class = "warlock", rarity = "rare", cost = 4,
    target_mode = "required", targets = function(ctx) return ctx:minions() end,
}
function card.on_play(ctx, self, target)
    cardlib.effects.damage(ctx, target, 3)
    ctx:continue_with("summon_pterrordaxes")
end
function card.summon_pterrordaxes(ctx, self)
    local floor = 0
    for _, entity in ipairs(ctx:minions()) do if entity > floor then floor = entity end end
    ctx:set_data(self, "feeding_floor", floor)
    for _ = 1, 3 do ctx:summon(ctx:controller(self), "UNG_834t1") end
    ctx:continue_with("choose_feeding_adapt")
end
function card.choose_feeding_adapt(ctx, self)
    local floor = ctx:get_data(self, "feeding_floor")
    local found = false
    for _, entity in ipairs(ctx:friendly_minions(self)) do
        if entity > floor and ctx:entity(entity).card_id == "UNG_834t1" then found = true break end
    end
    if found then ctx:discover_cards(ctx:controller(self), "Adapt", options, 3, "adapted_pterrordaxes") end
end
function card.adapted_pterrordaxes(ctx, self, choice)
    local floor = ctx:get_data(self, "feeding_floor")
    for _, entity in ipairs(ctx:friendly_minions(self)) do
        if entity > floor and ctx:entity(entity).card_id == "UNG_834t1" then apply(ctx, entity, choice) end
    end
end
card.tokens = {{ id = "UNG_834t1", name = "Pterrordax", text = "", set = "UNGORO", type = "minion", class = "warlock", collectible = false, cost = 1, attack = 1, health = 1, tags = { "beast" } }}
return card
