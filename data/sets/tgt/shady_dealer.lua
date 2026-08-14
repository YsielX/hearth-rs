local function is_pirate(ctx, entity)
    for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags) do
        if tag == "pirate" or tag == "all" then return true end
    end
    return false
end

return {
    api_version = 1,
    id = "AT_032",
    name = "Shady Dealer",
    text = "<b>Battlecry:</b> If you have a Pirate, gain +1/+1.",
    set = "TGT",
    type = "minion",
    class = "rogue",
    rarity = "rare",
    cost = 3,
    attack = 4,
    health = 3,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            if minion ~= self and is_pirate(ctx, minion) then
                ctx:buff(self, 1, 1)
                return
            end
        end
    end,
}
