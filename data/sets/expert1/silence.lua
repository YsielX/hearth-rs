return {
    api_version = 1,
    id = "EX1_332",
    name = "Silence",
    text = "<b>Silence</b> a minion.",
    set = "EXPERT1",
    type = "spell",
    class = "priest",
    cost = 0,
    keywords = { "silence" },
    target_mode = "required",
    targets = function(ctx, self)
        return ctx:minions()
    end,
    on_play = function(ctx, self, target)
        ctx:silence(target)
    end,
}
