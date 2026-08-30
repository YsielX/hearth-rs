return {
    api_version = 1,
    id = "AV_121", rarity = "common",
    name = "Gnome Private",
    text = "[x]<b>Honorable Kill:</b> Gain\n+2 Attack.",
    set = "ALTERAC_VALLEY",
    type = "minion",
    cost = 1,
    attack = 1,
    health = 3,
    keywords = { "honorable_kill" },
    on_honorable_kill = function(ctx, self)
        cardlib.effects.buff(ctx, self, 2, 0)
    end,
}
