local function dormant(ctx, entity)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do if keyword == "dormant" then return true end end
    return false
end
return { api_version = 1, id = "UNG_919", name = "Swamp King Dred",
    text = "After your opponent plays a minion, attack it.", set = "UNGORO", type = "minion",
    class = "hunter", rarity = "legendary", cost = 6, attack = 9, health = 9, tags = { "beast" },
    triggers = {{ event = "minion_played", timing = "after", active_zones = { "board" },
        condition = function(ctx, self, event)
            return event.player == ctx:opponent(ctx:controller(self))
                and ctx:entity(self).zone == "board" and ctx:entity(event.entity).zone == "board"
                and not dormant(ctx, self) and not dormant(ctx, event.entity)
        end,
        effect = function(ctx, self, event) ctx:force_attack(self, event.entity) end }} }
