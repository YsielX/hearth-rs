local card = {
    api_version = 1,
    id = "ICC_834",
    name = "Scourgelord Garrosh",
    text = "<b>Battlecry</b>: Equip a 4/3 Shadowmourne that also damages adjacent minions.",
    set = "ICECROWN",
    type = "hero",
    class = "warrior",
    cost = 8,
    health = 30,
    armor = 5,
    hero_power = "ICC_834h",
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    ctx:equip_weapon(ctx:controller(self), "ICC_834w")
end

card.tokens = {
    {
        id = "ICC_834w", name = "Shadowmourne",
        text = "Also damages the minions next to whomever your hero attacks.",
        set = "ICECROWN", type = "weapon", class = "warrior",
        cost = 8, attack = 4, health = 3,
        triggers = {
            {
                event = "attack", timing = "after", active_zones = { "weapon" },
                condition = function(ctx, self, event)
                    local player = ctx:controller(self)
                    return event.attacker == ctx:player(player).hero
                        and ctx:entity(event.defender).type == "minion"
                end,
                effect = function(ctx, self, event)
                    local amount = ctx:entity(self).attack
                    for _, adjacent in ipairs(ctx:adjacent_minions(event.defender)) do
                        ctx:damage(adjacent, amount)
                    end
                end,
            },
        },
    },
}

return card
