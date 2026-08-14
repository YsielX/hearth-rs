return {
    api_version = 1,
    id = "EX1_049",
    name = "Youthful Brewmaster",
    text = "<b>Battlecry:</b> Return a friendly minion from the battlefield to your hand.",
    set = "EXPERT1",
    type = "minion",
    cost = 2,
    attack = 3,
    health = 2,
    keywords = { "battlecry" },
    target_mode = "required_if_available",
    targets = function(ctx, self) return ctx:friendly_minions(self) end,
    on_battlecry = function(ctx, self, target)
        if target ~= nil then ctx:move(target, "hand") end
    end,
}
