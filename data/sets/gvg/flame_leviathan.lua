local function is_mech(ctx, entity)
    if ctx:entity(entity).type ~= "minion" then return false end
    local definition = ctx:card_definition(ctx:entity(entity).card_id)
    for _, tag in ipairs(definition.tags) do
        if tag == "mech" then return true end
    end
    return false
end

return {
    api_version = 1,
    id = "GVG_007",
    name = "Flame Leviathan",
    text = "[x]<b>Rush</b>\nWhen you draw this, deal\n2 damage to all characters\nexcept Mechs.",
    set = "GVG",
    type = "minion",
    class = "mage",
    rarity = "legendary",
    cost = 7,
    attack = 7,
    health = 7,
    tags = { "mech" },
    keywords = { "rush" },
    triggers = {
        {
            event = "card_drawn",
            timing = "after",
            active_zones = { "hand" },
            condition = function(ctx, self, event) return event.entity == self end,
            effect = function(ctx)
                local targets = {}
                for _, character in ipairs(ctx:characters()) do
                    if not is_mech(ctx, character) then targets[#targets + 1] = character end
                end
                ctx:damage_all(targets, 2)
            end,
        },
    },
}
