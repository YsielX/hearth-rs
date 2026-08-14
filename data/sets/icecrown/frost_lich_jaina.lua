local card = {
    api_version = 1,
    id = "ICC_833",
    name = "Frost Lich Jaina",
    text = "[x]<b>Battlecry:</b> Summon a\n3/6 Water Elemental.\nYour Elementals have\n<b>Lifesteal</b> this game.",
    set = "ICECROWN",
    type = "hero",
    class = "mage",
    cost = 9,
    health = 30,
    armor = 5,
    hero_power = "ICC_833h",
    keywords = { "battlecry" },
    auras = {
        {
            active_zones = { "hero", "removed" },
            keywords = { "lifesteal" },
            targets = function(ctx, self)
                local result = {}
                for _, minion in ipairs(ctx:friendly_minions(self)) do
                    local definition = ctx:card_definition(ctx:entity(minion).card_id)
                    for _, tag in ipairs(definition.tags) do
                        if tag == "elemental" then result[#result + 1] = minion break end
                    end
                end
                return result
            end,
        },
    },
}

function card.on_battlecry(ctx, self)
    ctx:summon(ctx:controller(self), "ICC_833t")
end

card.tokens = {
    {
        id = "ICC_833t", name = "Water Elemental",
        text = "<b>Freeze</b> any character damaged by this minion.",
        set = "ICECROWN", type = "minion", class = "mage",
        cost = 4, attack = 3, health = 6, tags = { "elemental" },
        keywords = { "freeze" },
        triggers = {
            {
                event = "damaged", timing = "after", active_zones = { "board" },
                condition = function(ctx, self, event)
                    return event.source == self and event.amount > 0
                end,
                effect = function(ctx, self, event) ctx:freeze(event.target) end,
            },
        },
    },
}

return card
