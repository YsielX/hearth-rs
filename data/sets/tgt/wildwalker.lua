local function friendly_beasts(ctx, self)
    local result = {}
    for _, minion in ipairs(ctx:friendly_minions(self)) do
        for _, tag in ipairs(ctx:card_definition(ctx:entity(minion).card_id).tags) do
            if tag == "beast" or tag == "all" then result[#result + 1] = minion break end
        end
    end
    return result
end

return {
    api_version = 1, id = "AT_040", name = "Wildwalker",
    text = "<b>Battlecry:</b> Give a friendly Beast +3 Health.",
    set = "TGT", type = "minion", class = "druid", rarity = "common",
    cost = 4, attack = 4, health = 4, keywords = { "battlecry" },
    target_mode = "required_if_available", targets = friendly_beasts,
    on_battlecry = function(ctx, self, target)
        if target ~= nil then ctx:buff(target, 0, 3) end
    end,
}
