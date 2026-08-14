local card = {
    api_version = 1, id = "OG_195", name = "Wisps of the Old Gods",
    text = "<b>Choose One -</b> Summon seven 1/1 Wisps; or Give your minions +2/+2.",
    set = "OG", type = "spell", class = "druid", rarity = "epic",
    spell_school = "nature", cost = 7, keywords = { "choose_one" },
}

local function summon(ctx, self)
    local player = ctx:controller(self)
    for _ = 1, 7 do ctx:summon(player, "OG_195c") end
end
local function buff(ctx, self)
    for _, minion in ipairs(ctx:board(ctx:controller(self))) do
        local entity = ctx:entity(minion)
        local dormant = false
        for _, keyword in ipairs(entity.keywords) do
            if keyword == "dormant" then dormant = true break end
        end
        if entity.type == "minion" and not dormant then ctx:buff(minion, 2, 2) end
    end
end

function card.on_choose_one(ctx, self)
    ctx:choose_options(ctx:controller(self), "Choose One", {
        { label = "Summon seven 1/1 Wisps", value = 1 },
        { label = "Give your minions +2/+2", value = 2 },
    }, "chosen")
end
function card.chosen(ctx, self, choice)
    if choice == 1 then summon(ctx, self) else buff(ctx, self) end
end
function card.on_choose_multiple(ctx, self)
    summon(ctx, self)
    ctx:continue_with("buff_summoned_wisps")
end
function card.buff_summoned_wisps(ctx, self) buff(ctx, self) end

card.tokens = {{
    id = "OG_195c", name = "Wisp", text = "", set = "OG", type = "minion",
    class = "druid", cost = 0, attack = 1, health = 1, tags = { "undead" },
}}
return card
