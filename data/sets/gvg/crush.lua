return {
    api_version = 1,
    id = "GVG_052",
    name = "Crush",
    text = "Destroy a minion. If you have a damaged minion, this costs (4) less.",
    set = "GVG",
    type = "spell",
    class = "warrior",
    rarity = "epic",
    cost = 7,
    target_mode = "required",
    targets = function(ctx) return ctx:minions() end,
    on_play = function(ctx, self, target) ctx:destroy(target) end,
    auras = {
        {
            active_zones = { "hand" },
            cost = -4,
            targets = function(ctx, self)
                for _, minion in ipairs(ctx:friendly_minions(self)) do
                    if ctx:entity(minion).damage > 0 then return { self } end
                end
                return {}
            end,
        },
    },
}
