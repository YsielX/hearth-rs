local power = {
    api_version = 1,
    module_type = "hero_power",
    id = "ICC_832p",
    name = "Plague Lord",
    text = "<b>Choose One -</b>\n+$a3 Attack this turn;\nor Gain $d3 Armor.",
    set = "ICECROWN",
    class = "neutral",
    cost = 2,
    keywords = { "choose_one" },
}

function power.on_choose_one(ctx, self)
    ctx:choose_options(ctx:controller(self), "Choose One", {
        { label = "+3 Attack this turn", value = 1 },
        { label = "Gain 3 Armor", value = 2 },
    }, "chosen")
end

function power.chosen(ctx, self, choice)
    local player = ctx:controller(self)
    if choice == 1 then
        ctx:buff_until_end_of_turn(ctx:player(player).hero, 3, 0)
    else
        ctx:gain_armor(player, 3)
    end
end

return power
