return {
    api_version = 1,
    id = "GVG_060",
    name = "Quartermaster",
    text = "<b>Battlecry:</b> Give your Silver Hand Recruits +2/+2.",
    set = "GVG",
    type = "minion",
    class = "paladin",
    rarity = "epic",
    cost = 5,
    attack = 2,
    health = 5,
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            if ctx:entity(minion).card_id == "CS2_101t" then
                cardlib.effects.buff(ctx, minion, 2, 2)
            end
        end
    end,
}
