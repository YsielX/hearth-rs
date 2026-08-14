return {
    api_version = 1,
    id = "LOE_104",
    name = "Entomb",
    text = "Choose an enemy minion.\nShuffle it into your deck.",
    set = "LOE",
    type = "spell",
    class = "priest",
    rarity = "common",
    cost = 6,
    target_mode = "required",
    targets = function(ctx, self)
        local result = {}
        for _, entity in ipairs(ctx:enemy_characters(self)) do
            if ctx:entity(entity).type == "minion" then result[#result + 1] = entity end
        end
        return result
    end,
    on_play = function(ctx, self, target)
        ctx:shuffle_entity_into_deck(ctx:controller(self), target)
    end,
}
