return {
    api_version = 1,
    id = "BAR_025", rarity = "common",
    name = "Sunwell Initiate",
    text = "<b>Frenzy:</b> Gain <b>Divine Shield</b>.",
    set = "THE_BARRENS",
    type = "minion",
    cost = 3,
    attack = 3,
    health = 4,
    keywords = { "frenzy" },
    on_frenzy = function(ctx, self)
        ctx:grant_keyword(self, "divine_shield")
    end,
}
