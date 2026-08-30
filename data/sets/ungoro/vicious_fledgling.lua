local options = { "UNG_999t2", "UNG_999t3", "UNG_999t4", "UNG_999t5", "UNG_999t6", "UNG_999t7", "UNG_999t8", "UNG_999t10", "UNG_999t13", "UNG_999t14" }
local function apply(ctx, target, choice)
    if choice == "UNG_999t2" then ctx:attach_hook(target, "on_deathrattle", "UNG_999t2"); cardlib.effects.grant_keyword(ctx, target, "deathrattle")
    elseif choice == "UNG_999t3" then cardlib.effects.buff(ctx, target, 3, 0)
    elseif choice == "UNG_999t4" then cardlib.effects.buff(ctx, target, 0, 3)
    elseif choice == "UNG_999t5" then cardlib.effects.grant_keyword(ctx, target, "elusive")
    elseif choice == "UNG_999t6" then cardlib.effects.grant_keyword(ctx, target, "taunt")
    elseif choice == "UNG_999t7" then cardlib.effects.grant_keyword(ctx, target, "windfury")
    elseif choice == "UNG_999t8" then cardlib.effects.grant_keyword(ctx, target, "divine_shield")
    elseif choice == "UNG_999t10" then ctx:grant_keyword_until_next_turn(target, "stealth")
    elseif choice == "UNG_999t13" then cardlib.effects.grant_keyword(ctx, target, "poisonous")
    else cardlib.effects.buff(ctx, target, 1, 1) end
end
local card = { api_version = 1, id = "UNG_075", name = "Vicious Fledgling",
    text = "After this minion attacks a hero, <b>Adapt</b>.", set = "UNGORO", type = "minion",
    rarity = "rare", cost = 3, attack = 3, health = 3, tags = { "beast" },
    triggers = {{ event = "attack", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event) return event.attacker == self and ctx:entity(event.defender).type == "hero" end,
        effect = function(ctx, self) ctx:discover_cards(ctx:controller(self), "Adapt", options, 3, "adapted") end }} }
function card.adapted(ctx, self, choice) apply(ctx, self, choice) end
return card
