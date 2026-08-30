local card = {
    api_version = 1,
    id = "RLK_086",
    name = "Frostmourne",
    text = "<b>Deathrattle:</b> Summon every minion killed by this weapon.",
    set = "PATH_OF_ARTHAS",
    type = "weapon",
    class = "death_knight",
    rarity = "legendary",
    cost = 6,
    attack = 4,
    health = 3,
    keywords = { "deathrattle" },
}

card.triggers = {{
    event = "attack",
    timing = "after",
    active_zones = { "weapon" },
    condition = function(ctx, self, event)
        local player = ctx:controller(self)
        local defender = ctx:entity(event.defender)
        return event.attacker == ctx:player(player).hero
            and defender.type == "minion"
            and defender.health <= 0
    end,
    effect = function(ctx, self, event)
        local count = ctx:get_data(self, "frostmourne_kill_count") + 1
        ctx:set_data(self, "frostmourne_kill_count", count)
        ctx:set_data(self, "frostmourne_kill_" .. count, event.defender)
    end,
}}

function card.on_deathrattle(ctx, self)
    local player = ctx:controller(self)
    for index = 1, ctx:get_data(self, "frostmourne_kill_count") do
        local killed = ctx:get_data(self, "frostmourne_kill_" .. index)
        if killed ~= 0 then ctx:summon(player, ctx:entity(killed).card_id) end
    end
end

return card
