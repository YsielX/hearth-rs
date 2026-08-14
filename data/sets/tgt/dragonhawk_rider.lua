return {
    api_version = 1, id = "AT_083", name = "Dragonhawk Rider",
    text = "<b>Inspire:</b> Gain <b>Windfury</b>\nthis turn.",
    set = "TGT", type = "minion", rarity = "common", cost = 3, attack = 3, health = 3,
    keywords = { "inspire" },
    on_inspire = function(ctx, self)
        ctx:grant_keyword_until_end_of_turn(self, "windfury")
    end,
}
