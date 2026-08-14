return {
    api_version = 1,
    id = "WORK_002",
    name = "Busy-Bot",
    text = "<b>Battlecry:</b> Give your\n1-Attack minions +1/+1.",
    set = "ISLAND_VACATION",
    type = "minion",
    class = "paladin",
    cost = 2,
    attack = 3,
    health = 2,
    tags = { "mech" },
    keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        for _, minion in ipairs(ctx:friendly_minions(self)) do
            if ctx:entity(minion).attack == 1 then ctx:buff(minion, 1, 1) end
        end
    end,
}
