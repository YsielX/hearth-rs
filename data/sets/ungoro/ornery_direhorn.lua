local options = { "UNG_999t2", "UNG_999t3", "UNG_999t4", "UNG_999t5", "UNG_999t6", "UNG_999t7", "UNG_999t8", "UNG_999t10", "UNG_999t13", "UNG_999t14" }
local card = { api_version = 1, id = "UNG_925", name = "Ornery Direhorn", text = "<b>Taunt</b>\n<b>Battlecry:</b> <b>Adapt</b>.", set = "UNGORO", type = "minion", class = "warrior", rarity = "common", cost = 6, attack = 6, health = 6, tags = { "beast" }, keywords = { "taunt", "battlecry" } }
function card.on_battlecry(ctx, self) ctx:discover_cards(ctx:controller(self), "Adapt", options, 3, "adapted") end
function card.adapted(ctx, self, choice)
    if ctx:entity(self).zone ~= "board" then return end
    if choice == "UNG_999t2" then ctx:attach_hook(self, "on_deathrattle", "UNG_999t2"); ctx:grant_keyword(self, "deathrattle")
    elseif choice == "UNG_999t3" then ctx:buff(self, 3, 0)
    elseif choice == "UNG_999t4" then ctx:buff(self, 0, 3)
    elseif choice == "UNG_999t5" then ctx:grant_keyword(self, "elusive")
    elseif choice == "UNG_999t6" then ctx:grant_keyword(self, "taunt")
    elseif choice == "UNG_999t7" then ctx:grant_keyword(self, "windfury")
    elseif choice == "UNG_999t8" then ctx:grant_keyword(self, "divine_shield")
    elseif choice == "UNG_999t10" then ctx:grant_keyword_until_next_turn(self, "stealth")
    elseif choice == "UNG_999t13" then ctx:grant_keyword(self, "poisonous")
    elseif choice == "UNG_999t14" then ctx:buff(self, 1, 1) end
end
return card
