return {
    api_version = 1,
    id = "CFM_693",
    name = "Gadgetzan Ferryman",
    text = "<b>Combo:</b> Return a friendly minion to your hand.",
    set = "GANGS",
    type = "minion",
    class = "rogue",
    cost = 2,
    attack = 2,
    health = 3,
    keywords = { "combo" },
    target_mode = "required_if_available",

    targets = function(ctx, self)
        local player = ctx:controller(self)
        if ctx:cards_played_this_turn(player) == 0 then return {} end
        return ctx:friendly_minions(self)
    end,
    on_combo = function(ctx, self, target)
        if target ~= nil then ctx:move(target, "hand") end
    end,
}
