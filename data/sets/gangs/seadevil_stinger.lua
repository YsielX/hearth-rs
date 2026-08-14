local ACTIVE = "next_murloc_health_cost"
local PENDING = "next_murloc_health_cost_pending"
local card = {
    api_version = 1, id = "CFM_699", name = "Seadevil Stinger",
    text = "[x]<b>Battlecry:</b> The next Murloc\nyou play this turn costs\n Health instead of Mana.",
    set = "GANGS", type = "minion", class = "warlock", rarity = "rare",
    cost = 4, attack = 4, health = 2, tags = { "murloc" }, keywords = { "battlecry" },
    triggers = {{
        event = "minion_summoned", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.entity == self and ctx:get_player_data(ctx:controller(self), PENDING) > 0
        end,
        effect = function(ctx, self)
            local player = ctx:controller(self)
            local pending = ctx:get_player_data(player, PENDING)
            ctx:increment_player_data(player, ACTIVE, pending)
            ctx:set_player_data(player, PENDING, 0)
        end,
    }},
}
function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    ctx:increment_player_data(player, PENDING, 1)
    ctx:grant_player_keyword(player, "next_murloc_costs_health")
end
return card
