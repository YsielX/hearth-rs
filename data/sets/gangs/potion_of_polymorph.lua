local function is_dormant(ctx, entity)
    for _, keyword in ipairs(ctx:entity(entity).keywords) do
        if keyword == "dormant" then return true end
    end
    return false
end

return {
    api_version = 1,
    id = "CFM_620",
    name = "Potion of Polymorph",
    text = "<b>Secret:</b> After your opponent plays a minion, transform it into a\n1/1 Sheep.",
    set = "GANGS",
    type = "spell",
    class = "mage",
    rarity = "rare",
    spell_school = "arcane",
    cost = 3,
    keywords = { "secret" },
    triggers = {{
        event = "minion_played", timing = "after", active_zones = { "secret" },
        condition = function(ctx, self, event)
            return event.player == ctx:opponent(ctx:controller(self))
                and ctx:entity(event.entity).zone == "board"
                and not is_dormant(ctx, event.entity)
        end,
        effect = function(ctx, self, event)
            ctx:reveal_secret(self)
            cardlib.effects.transform(ctx, event.entity, "CS2_tk1")
        end,
    }},
}
