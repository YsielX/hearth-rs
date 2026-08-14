local function empower_totems(ctx, self)
    for _, minion in ipairs(ctx:friendly_minions(self)) do
        for _, tag in ipairs(ctx:card_definition(ctx:entity(minion).card_id).tags or {}) do
            if tag == "totem" or tag == "all" then ctx:buff(minion, 2, 0) break end
        end
    end
end

return {
    api_version = 1, id = "AT_049", name = "Thunder Bluff Valiant",
    text = "<b>Battlecry and Inspire:</b>\nGive your Totems\n+2 Attack.", set = "TGT", type = "minion",
    class = "shaman", rarity = "rare", cost = 5, attack = 3, health = 6,
    keywords = { "battlecry", "inspire" },
    on_battlecry = empower_totems,
    on_inspire = empower_totems,
}
