return {
    api_version = 1,
    id = "EX1_613",
    name = "Edwin VanCleef",
    text = "<b>Combo:</b> Gain +2/+2 for each other card you've played this turn.",
    set = "EXPERT1",
    type = "minion",
    rarity = "legendary",
    class = "rogue",
    cost = 3,
    attack = 2,
    health = 2,
    keywords = { "combo" },

    on_combo = function(ctx, self)
        local cards = ctx:entity(self).cards_played_before
        ctx:buff(self, cards * 2, cards * 2)
    end,
}
