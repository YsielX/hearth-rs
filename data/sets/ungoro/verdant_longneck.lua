local options = { "UNG_999t2", "UNG_999t3", "UNG_999t4", "UNG_999t5", "UNG_999t6", "UNG_999t7", "UNG_999t8", "UNG_999t10", "UNG_999t13", "UNG_999t14" }
local function apply(ctx, target, choice)
    if choice == "UNG_999t2" then ctx:attach_deathrattle(target, "UNG_999t2"); ctx:grant_keyword(target, "deathrattle")
    elseif choice == "UNG_999t3" then ctx:buff(target, 3, 0) elseif choice == "UNG_999t4" then ctx:buff(target, 0, 3)
    elseif choice == "UNG_999t5" then ctx:grant_keyword(target, "elusive") elseif choice == "UNG_999t6" then ctx:grant_keyword(target, "taunt")
    elseif choice == "UNG_999t7" then ctx:grant_keyword(target, "windfury") elseif choice == "UNG_999t8" then ctx:grant_keyword(target, "divine_shield")
    elseif choice == "UNG_999t10" then ctx:grant_keyword_until_next_turn(target, "stealth") elseif choice == "UNG_999t13" then ctx:grant_keyword(target, "poisonous")
    else ctx:buff(target, 1, 1) end
end
local card = { api_version = 1, id = "UNG_100", name = "Verdant Longneck", text = "<b>Battlecry:</b> <b>Adapt</b>.",
    set = "UNGORO", type = "minion", class = "druid", rarity = "common", cost = 5,
    attack = 5, health = 5, tags = { "beast" }, keywords = { "battlecry" } }
function card.on_battlecry(ctx, self) ctx:discover_cards(ctx:controller(self), "Adapt", options, 3, "adapted") end
function card.adapted(ctx, self, choice) apply(ctx, self, choice) end
return card
