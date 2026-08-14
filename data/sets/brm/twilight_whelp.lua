local function is_dragon(ctx, entity)
    local definition = ctx:card_definition(ctx:entity(entity).card_id)
    for _, tag in ipairs(definition.tags) do
        if tag == "dragon" then return true end
    end
    return false
end

return {
    api_version = 1,
    id = "BRM_004",
    name = "Twilight Whelp",
    text = "<b>Battlecry:</b> If you're holding a Dragon, gain +2 Health.",
    set = "BRM",
    type = "minion",
    class = "priest",
    rarity = "common",
    cost = 1,
    attack = 2,
    health = 1,
    tags = { "dragon" },
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
            if is_dragon(ctx, entity) then
                ctx:buff(self, 0, 2)
                return
            end
        end
    end,
    tokens = {
        {
            id = "BRM_004t",
            name = "Whelp",
            text = "",
            set = "BRM",
            type = "minion",
            cost = 1,
            attack = 1,
            health = 1,
        },
    },
}
