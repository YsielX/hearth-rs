return {
    api_version = 1,
    id = "KAR_013",
    name = "Purify",
    text = "<b>Silence</b> a friendly minion. Draw a card.",
    set = "KARA",
    type = "spell",
    class = "priest",
    rarity = "common",
    spell_school = "holy",
    cost = 2,
    target_mode = "required",
    targets = function(ctx, self) return ctx:friendly_minions(self) end,
    on_play = function(ctx, self, target)
        ctx:silence(target)
        ctx:draw(ctx:controller(self), 1)
    end,
}
