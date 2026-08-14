local options = { "UNG_999t2", "UNG_999t3", "UNG_999t4", "UNG_999t5", "UNG_999t6", "UNG_999t7", "UNG_999t8", "UNG_999t10", "UNG_999t13", "UNG_999t14" }
local function apply(ctx, target, choice)
    if choice == "UNG_999t2" then ctx:attach_hook(target, "on_deathrattle", "UNG_999t2"); ctx:grant_keyword(target, "deathrattle")
    elseif choice == "UNG_999t3" then ctx:buff(target, 3, 0)
    elseif choice == "UNG_999t4" then ctx:buff(target, 0, 3)
    elseif choice == "UNG_999t5" then ctx:grant_keyword(target, "elusive")
    elseif choice == "UNG_999t6" then ctx:grant_keyword(target, "taunt")
    elseif choice == "UNG_999t7" then ctx:grant_keyword(target, "windfury")
    elseif choice == "UNG_999t8" then ctx:grant_keyword(target, "divine_shield")
    elseif choice == "UNG_999t10" then ctx:grant_keyword_until_next_turn(target, "stealth")
    elseif choice == "UNG_999t13" then ctx:grant_keyword(target, "poisonous")
    else ctx:buff(target, 1, 1) end
end
local card = { api_version = 1, id = "UNG_002", name = "Volcanosaur", text = "<b>Battlecry:</b> <b>Adapt</b>, then <b>Adapt</b>.",
    set = "UNGORO", type = "minion", rarity = "rare", cost = 6, attack = 5, health = 6,
    tags = { "elemental", "beast" }, keywords = { "battlecry" } }
function card.on_battlecry(ctx, self) ctx:set_data(self, "adapts", 2); ctx:continue_with("choose_adapt") end
function card.choose_adapt(ctx, self)
    if ctx:get_data(self, "adapts") > 0 then ctx:discover_cards(ctx:controller(self), "Adapt", options, 3, "adapted") end
end
function card.adapted(ctx, self, choice)
    apply(ctx, self, choice); ctx:set_data(self, "adapts", ctx:get_data(self, "adapts") - 1); ctx:continue_with("choose_adapt")
end
return card
