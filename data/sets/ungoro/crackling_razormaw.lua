local options = { "UNG_999t2", "UNG_999t3", "UNG_999t4", "UNG_999t5", "UNG_999t6", "UNG_999t7", "UNG_999t8", "UNG_999t10", "UNG_999t13", "UNG_999t14" }
local function beast(ctx, target)
    for _, tag in ipairs(ctx:card_definition(ctx:entity(target).card_id).tags or {}) do if tag == "beast" or tag == "all" then return true end end
    return false
end
local function apply(ctx, target, choice)
    if choice == "UNG_999t2" then ctx:attach_hook(target, "on_deathrattle", "UNG_999t2"); ctx:grant_keyword(target, "deathrattle")
    elseif choice == "UNG_999t3" then ctx:buff(target, 3, 0) elseif choice == "UNG_999t4" then ctx:buff(target, 0, 3)
    elseif choice == "UNG_999t5" then ctx:grant_keyword(target, "elusive") elseif choice == "UNG_999t6" then ctx:grant_keyword(target, "taunt")
    elseif choice == "UNG_999t7" then ctx:grant_keyword(target, "windfury") elseif choice == "UNG_999t8" then ctx:grant_keyword(target, "divine_shield")
    elseif choice == "UNG_999t10" then ctx:grant_keyword_until_next_turn(target, "stealth") elseif choice == "UNG_999t13" then ctx:grant_keyword(target, "poisonous")
    else ctx:buff(target, 1, 1) end
end
local card = { api_version = 1, id = "UNG_915", name = "Crackling Razormaw",
    text = "<b>Battlecry:</b> <b>Adapt</b> a friendly Beast.", set = "UNGORO", type = "minion",
    class = "hunter", rarity = "common", cost = 2, attack = 3, health = 2, tags = { "beast" },
    keywords = { "battlecry" }, target_mode = "required_if_available",
    targets = function(ctx, self)
        local result = {}; for _, minion in ipairs(ctx:friendly_minions(self)) do if minion ~= self and beast(ctx, minion) then result[#result + 1] = minion end end; return result
    end }
function card.on_battlecry(ctx, self, target)
    if target then ctx:set_data(self, "adapt_target", target); ctx:discover_cards(ctx:controller(self), "Adapt", options, 3, "adapted") end
end
function card.adapted(ctx, self, choice)
    local target = ctx:get_data(self, "adapt_target"); if target ~= 0 and ctx:entity(target).zone == "board" then apply(ctx, target, choice) end
end
return card
