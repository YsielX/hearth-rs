local function enemies(ctx, self)
    local result = {}
    local controller = ctx:controller(self)
    for _, character in ipairs(ctx:characters()) do
        if ctx:controller(character) ~= controller then
            result[#result + 1] = character
        end
    end
    return result
end

return {
    api_version = 1,
    id = "RLK_038",
    name = "Icy Touch",
    text = "Deal $2 damage to an enemy and <b>Freeze</b> it.",
    set = "PATH_OF_ARTHAS",
    type = "spell",
    class = "death_knight",
    cost = 1,
    target_mode = "required",
    targets = enemies,
    on_play = function(ctx, self, target)
        ctx:damage(target, 2)
        ctx:freeze(target)
    end,
}
