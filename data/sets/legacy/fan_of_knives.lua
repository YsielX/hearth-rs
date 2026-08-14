return {
    api_version = 1,
    id = "EX1_129",
    name = "Fan of Knives",
    text = "Deal $1 damage to all enemy minions. Draw a card.",
    set = "LEGACY",
    type = "spell",
    class = "rogue",
    cost = 2,
    on_play = function(ctx, self)
        local targets = {}
        for _, entity in ipairs(ctx:minions()) do
            if ctx:controller(entity) ~= ctx:controller(self) then
                targets[#targets + 1] = entity
            end
        end
        ctx:damage_all(targets, 1)
        ctx:draw(ctx:controller(self), 1)
    end,
}
