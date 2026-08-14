return {
    api_version = 1,
    id = "GVG_031",
    name = "Recycle",
    text = "Shuffle an enemy minion into your opponent's deck.",
    set = "GVG",
    type = "spell",
    class = "druid",
    spell_school = "nature",
    rarity = "rare",
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
        ctx:move(target, "deck_random")
    end,
}
