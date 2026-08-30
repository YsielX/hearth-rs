local function taunt(ctx, entity)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do if keyword == "taunt" then return true end end
    return false
end
local card = {
    api_version = 1, id = "UNG_934", name = "Fire Plume's Heart",
    text = "[x]<b>Quest:</b> Play\n7 <b>Taunt</b> minions.\n<b>Reward:</b> Sulfuras.",
    set = "UNGORO", type = "spell", class = "warrior", rarity = "legendary", cost = 1, keywords = { "quest" },
    triggers = {
    {
        event = "card_played", timing = "before", active_zones = { "secret" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self) and ctx:get_data(self, "completed") == 0 and ctx:entity(event.entity).type == "minion"
        end,
        effect = function(ctx, self, event) ctx:set_data(self, "played_taunt", taunt(ctx, event.entity) and 1 or 0) end,
    }, {
        event = "card_played", timing = "after", active_zones = { "secret" },
        condition = function(ctx, self, event)
            return event.player == ctx:controller(self) and ctx:get_data(self, "completed") == 0 and ctx:get_data(self, "played_taunt") == 1
        end,
        effect = function(ctx, self)
            ctx:set_data(self, "played_taunt", 0)
            local progress = ctx:get_data(self, "progress") + 1
            ctx:set_data(self, "progress", progress)
            if progress >= 7 then ctx:set_data(self, "completed", 1); ctx:reveal_secret(self); cardlib.effects.give_card(ctx, ctx:controller(self), "UNG_934t1") end
        end,
    }},
}
local sulfuras = { id = "UNG_934t1", name = "Sulfuras", text = "<b>Battlecry:</b> Your Hero Power becomes 'Deal 8 damage to a random enemy.'", set = "UNGORO", type = "weapon", class = "warrior", collectible = false, cost = 3, attack = 4, health = 2 }
function sulfuras.on_play(ctx, self) ctx:replace_hero_power(ctx:controller(self), "UNG_934t2") end
local power = { id = "UNG_934t2", name = "DIE, INSECT!", text = "Deal $8 damage to a random enemy.", set = "UNGORO", type = "hero_power", collectible = false, cost = 2 }
function power.on_play(ctx, self)
    local targets = ctx:enemy_characters(self)
    if #targets > 0 then ctx:random_entity(targets, "deal_insect_damage") end
end
function power.deal_insect_damage(ctx, self, target) cardlib.effects.damage(ctx, target, 8) end
card.tokens = { sulfuras, power }
return card
