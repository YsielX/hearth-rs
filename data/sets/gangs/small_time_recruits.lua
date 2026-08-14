local card = {
    api_version = 1, id = "CFM_905", name = "Small-Time Recruits",
    text = "[x]Draw three 1-Cost\nminions from your deck.",
    set = "GANGS", type = "spell", class = "paladin", rarity = "epic", cost = 3,
}
local function draw_next(ctx, self, remaining)
    if remaining <= 0 then return end
    local candidates = {}
    for _, entity in ipairs(ctx:deck(ctx:controller(self))) do
        local info = ctx:entity(entity)
        if info.type == "minion" and info.cost == 1 then candidates[#candidates + 1] = entity end
    end
    if #candidates > 0 then
        ctx:set_data(self, "recruits_remaining", remaining)
        ctx:random_entity(candidates, "draw_small_time_recruit")
    end
end
function card.on_play(ctx, self) draw_next(ctx, self, 3) end
function card.draw_small_time_recruit(ctx, self, target)
    local remaining = ctx:get_data(self, "recruits_remaining") or 1
    ctx:draw_entity(ctx:controller(self), target)
    ctx:continue_with_value("continue_small_time_recruits", remaining - 1)
end
function card.continue_small_time_recruits(ctx, self, remaining) draw_next(ctx, self, remaining) end
return card
